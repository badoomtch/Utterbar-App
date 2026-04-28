import { useEffect, useState } from 'react';
import { AlertTriangle } from 'lucide-react';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { Button } from '@/components/common/Button';
import { useSettings } from '@/state/settings';
import { useUpdate } from '@/state/update';
import { invoke } from '@/lib/ipc';
import { cn } from '@/lib/cn';

const UPDATE_CHANNELS = [
  { id: 'stable' as const, label: 'Stable' },
  { id: 'beta' as const, label: 'Beta' },
];

/**
 * Settings → Updates.
 *
 * Three blocks:
 *   1. A rose-tinted warning when the user is on the beta channel.
 *      Beta builds can break the install or lose data; the warning
 *      makes that explicit before the user notices the hard way.
 *   2. Channel selector, auto-check toggle, and a manual "check now"
 *      button.
 *   3. Release notes for the version currently installed. Bundled
 *      into the binary at compile time via `current_release_notes`
 *      on the Rust side, rendered through the small inline markdown
 *      parser at the bottom of this file.
 */
export function UpdatesSettings() {
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);

  const checking = useUpdate((x) => x.checking);
  const lastManualOutcome = useUpdate((x) => x.lastManualOutcome);
  const updateInfo = useUpdate((x) => x.info);
  const runCheck = useUpdate((x) => x.check);

  const [notes, setNotes] = useState<string | null>(null);
  useEffect(() => {
    invoke<string | null>('current_release_notes')
      .then((md) => setNotes(md ?? null))
      .catch(() => setNotes(null));
  }, []);

  if (!s) return null;
  const g = s.general as Record<string, unknown>;
  // Same shape as GeneralSettings' local helper — patch a single
  // general-settings key without clobbering the rest of the section.
  const update = (key: string, value: unknown) =>
    patch({ general: { ...s.general, [key]: value } });
  const channel = (s.advanced as { update_channel?: 'stable' | 'beta' }).update_channel ?? 'stable';
  const onBeta = channel === 'beta';
  const version = updateInfo?.current ?? '';

  return (
    <div className="mx-auto w-full max-w-[880px] px-6 py-6">
      <PageHeader
        title="Updates"
        subtitle="Choose your channel, see what changed in this version, and check for new releases."
      />

      <div className="mt-6 space-y-4">
        {onBeta && (
          // Single, prominent warning on the beta channel. Borrows the
          // accent-rose colour family used elsewhere for "stop and read"
          // surfaces (delete confirms, error toasts) so it sits above
          // the channel toggle visually without yelling.
          <div className="rounded-input border border-rose-500/40 bg-rose-500/10 p-4">
            <div className="flex items-start gap-3">
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-rose-400" />
              <div className="text-[13px] leading-relaxed text-fg">
                <div className="font-medium text-rose-300">You're on the beta channel.</div>
                <div className="mt-1 text-muted">
                  Beta builds are pre-release. They may break your install, lose data, or
                  introduce new bugs. Switch to Stable below if you'd rather only see
                  releases that have soaked through testing first.
                </div>
              </div>
            </div>
          </div>
        )}

        <Card
          title="Channel"
          description="Stable is the public-facing channel. Only releases that have soaked through beta land there. Beta receives pre-release builds for testing."
        >
          <Row label="Update channel">
            <div className="flex gap-2">
              {UPDATE_CHANNELS.map((c) => {
                const current = channel === c.id;
                return (
                  <button
                    key={c.id}
                    onClick={() =>
                      patch({
                        advanced: {
                          ...s.advanced,
                          update_channel: c.id,
                        },
                      })
                    }
                    className={cn('ub-chip', current && 'ub-chip--active')}
                  >
                    {c.label}
                  </button>
                );
              })}
            </div>
          </Row>
          <Row
            label="Check for updates automatically"
            description="Checks the update feed on launch and shows a toast if a newer version is available. No auto-install, you always click Install."
          >
            <Toggle
              checked={Boolean(g.check_updates_automatically)}
              onChange={(v) => void update('check_updates_automatically', v)}
            />
          </Row>
          <Row
            label="Check now"
            description={
              lastManualOutcome === 'available' && updateInfo?.available
                ? `${updateInfo.current} → ${updateInfo.available}, see the toast to install.`
                : lastManualOutcome === 'up-to-date'
                  ? `You're up to date.`
                  : lastManualOutcome === 'failed'
                    ? "Couldn't reach the update server. Check your connection and try again."
                    : 'Asks the update server right now, independent of the launch check.'
            }
          >
            <Button
              variant="ghost"
              size="sm"
              noLed
              onClick={() => void runCheck({ manual: true })}
              disabled={checking}
            >
              {checking ? 'Checking…' : 'Check now'}
            </Button>
          </Row>
        </Card>

        <Card
          title={version ? `What's new in ${version}` : "What's new in this version"}
          description="Release notes for the version you're running. Older versions' notes live in the GitHub releases archive."
        >
          {notes ? <ReleaseNotes md={notes} /> : (
            <div className="px-4 py-3 text-sm text-muted">
              No release notes bundled for this version. (Probably running a dev build.)
            </div>
          )}
        </Card>
      </div>
    </div>
  );
}

/**
 * Tiny markdown renderer scoped to what release notes use:
 *   - `# heading` / `## heading`
 *   - paragraphs (blank-line separated)
 *   - bullet lists (`- ` prefix)
 *   - bold (`**...**`)
 *   - inline code (`` `...` ``)
 *
 * Deliberately not pulling in `react-markdown` (60 KB+ + remark deps)
 * for ~five formatting features we control entirely. If notes ever
 * grow tables / images / nested lists the right move is the proper
 * library; until then this is honest.
 */
function ReleaseNotes({ md }: { md: string }) {
  const blocks = parseBlocks(md);
  return (
    <div className="space-y-3 px-4 py-3 text-[13px] leading-relaxed text-fg">
      {blocks.map((b, i) => {
        switch (b.kind) {
          case 'h1':
            return <h2 key={i} className="text-lg font-medium text-fg">{renderInline(b.text)}</h2>;
          case 'h2':
            return <h3 key={i} className="mt-2 text-sm font-medium text-fg">{renderInline(b.text)}</h3>;
          case 'p':
            return <p key={i} className="text-muted">{renderInline(b.text)}</p>;
          case 'list':
            return (
              <ul key={i} className="ml-4 list-disc space-y-1 text-muted">
                {b.items.map((item, j) => (
                  <li key={j}>{renderInline(item)}</li>
                ))}
              </ul>
            );
        }
      })}
    </div>
  );
}

type Block =
  | { kind: 'h1'; text: string }
  | { kind: 'h2'; text: string }
  | { kind: 'p'; text: string }
  | { kind: 'list'; items: string[] };

function parseBlocks(md: string): Block[] {
  const blocks: Block[] = [];
  // Split into paragraphs by blank lines, then classify each.
  const paras = md.split(/\n{2,}/).map((p) => p.trim()).filter(Boolean);
  for (const para of paras) {
    const lines = para.split('\n');
    if (para.startsWith('# ')) {
      blocks.push({ kind: 'h1', text: para.slice(2).trim() });
    } else if (para.startsWith('## ')) {
      blocks.push({ kind: 'h2', text: para.slice(3).trim() });
    } else if (lines.every((l) => l.startsWith('- '))) {
      blocks.push({ kind: 'list', items: lines.map((l) => l.slice(2).trim()) });
    } else {
      // Collapse newlines inside a paragraph to single spaces so
      // hard-wrapped markdown reads as flowing prose.
      blocks.push({ kind: 'p', text: lines.join(' ') });
    }
  }
  return blocks;
}

/**
 * Render inline markdown — bold and code — into React fragments. Order
 * matters: code is greedier than bold (it can contain `**`) so we
 * tokenise code first and only run bold on the non-code spans.
 */
function renderInline(text: string): React.ReactNode {
  // Split on code spans first.
  const codeParts = text.split(/(`[^`]+`)/);
  return codeParts.map((part, i) => {
    if (part.startsWith('`') && part.endsWith('`')) {
      return (
        <code key={i} className="rounded bg-bg-2/80 px-1 py-0.5 font-mono text-[12px] text-fg">
          {part.slice(1, -1)}
        </code>
      );
    }
    // Bold inside non-code spans.
    const boldParts = part.split(/(\*\*[^*]+\*\*)/);
    return boldParts.map((bp, j) => {
      if (bp.startsWith('**') && bp.endsWith('**')) {
        return (
          <strong key={`${i}-${j}`} className="font-medium text-fg">
            {bp.slice(2, -2)}
          </strong>
        );
      }
      return <span key={`${i}-${j}`}>{bp}</span>;
    });
  });
}
