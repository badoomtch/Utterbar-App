import { useEffect, useState } from 'react';
import { AlertTriangle } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { Button } from '@/components/common/Button';
import { useSettings } from '@/state/settings';
import { useUpdate } from '@/state/update';
import { invoke } from '@/lib/ipc';
import { cn } from '@/lib/cn';

// Stable id registry. ids match the `update_channel` field's
// serialised values on the Rust side, so the patch wiring continues
// to write `'stable' | 'beta'`. Visible labels resolve via t() at
// render time.
const UPDATE_CHANNEL_IDS = ['stable', 'beta'] as const;

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
  const { t } = useTranslation('settings');
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

  // The check-now row's helper text reflects the last manual outcome.
  // updateInfo.current and updateInfo.available are backend-emitted
  // version strings and stay verbatim — they're interpolated into the
  // `description_available` template at render time, never catalogued.
  const checkNowDescription =
    lastManualOutcome === 'available' && updateInfo?.available
      ? t('updates.channel.check_now.description_available', {
          current: updateInfo.current,
          available: updateInfo.available,
        })
      : lastManualOutcome === 'up-to-date'
        ? t('updates.channel.check_now.description_up_to_date')
        : lastManualOutcome === 'failed'
          ? t('updates.channel.check_now.description_failed')
          : t('updates.channel.check_now.description_idle');

  return (
    <div className="mx-auto w-full max-w-[880px] px-6 py-6">
      <PageHeader title={t('updates.title')} subtitle={t('updates.subtitle')} />

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
                <div className="font-medium text-rose-300">
                  {t('updates.beta_warning.heading')}
                </div>
                <div className="mt-1 text-muted">{t('updates.beta_warning.body')}</div>
              </div>
            </div>
          </div>
        )}

        <Card
          title={t('updates.channel.title')}
          description={t('updates.channel.description')}
        >
          <Row label={t('updates.channel.row_label')}>
            <div className="flex gap-2">
              {UPDATE_CHANNEL_IDS.map((id) => {
                const current = channel === id;
                return (
                  <button
                    key={id}
                    onClick={() =>
                      patch({
                        advanced: {
                          ...s.advanced,
                          update_channel: id,
                        },
                      })
                    }
                    className={cn('ub-chip', current && 'ub-chip--active')}
                  >
                    {t(`updates.channel.options.${id}` as const)}
                  </button>
                );
              })}
            </div>
          </Row>
          <Row
            label={t('updates.channel.auto_check.label')}
            description={t('updates.channel.auto_check.description')}
          >
            <Toggle
              checked={Boolean(g.check_updates_automatically)}
              onChange={(v) => void update('check_updates_automatically', v)}
            />
          </Row>
          <Row
            label={t('updates.channel.check_now.label')}
            description={checkNowDescription}
          >
            <Button
              variant="ghost"
              size="sm"
              noLed
              onClick={() => void runCheck({ manual: true })}
              disabled={checking}
            >
              {checking
                ? t('updates.channel.check_now.checking')
                : t('updates.channel.check_now.button')}
            </Button>
          </Row>
        </Card>

        <Card
          // The running version is a backend-emitted string and stays
          // verbatim; only the surrounding "What's new in" prose is
          // catalogued. When the version isn't yet known, fall back to
          // the version-less title.
          title={
            version
              ? t('updates.release_notes.title_with_version', { version })
              : t('updates.release_notes.title_fallback')
          }
          description={t('updates.release_notes.description')}
        >
          {notes ? <ReleaseNotes md={notes} /> : (
            <div className="px-4 py-3 text-sm text-muted">
              {t('updates.release_notes.empty')}
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
