import { useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Plus, Zap } from 'lucide-react';

import { Button } from '@/components/common/Button';
import { Card, Row } from '@/components/common/Card';
import { Input } from '@/components/common/Input';
import { Kbd } from '@/components/common/Kbd';
import { PageHeader } from '@/components/common/PageHeader';
import { invoke } from '@/lib/ipc';
import { useSettings } from '@/state/settings';

type Vocab = { id: string; word: string; hint: string | null; enabled: boolean; weight: number };
type Replacement = {
  id: string;
  pattern: string;
  replacement: string;
  enabled: boolean;
  is_regex: boolean;
  case_sens: boolean;
  sort_order: number;
};

/**
 * Vocabulary — the local-only dictionary and post-ASR replacement rules.
 *
 * Per DESIGN_SYSTEM.md §6.1 inputs sit on bg-0 inside a bg-1 Card — the
 * "type into this hole" reading depends on the input being one step
 * BELOW the surrounding surface. All inputs here use the `.ub-input`
 * class via the Input primitive.
 *
 * Row list rhythm stays hairline-only (§4.3); no rail per row since
 * individual vocab entries aren't a selection surface.
 */
/**
 * Sort order for the word list.
 *
 * v0.6.7 adds explicit user-facing sort choice because the backend list
 * comes back in insertion order. With only a handful of entries that
 * reads fine, but once you have 50+ entries you can't find anything
 * without scanning the whole list. Alphabetical is the default because
 * "I know what I typed, where is it?" is the dominant workflow; "recent"
 * is a secondary option for the "I just added something, was the spelling
 * what I think?" case.
 */
type WordSort = 'alpha' | 'recent';

export function VocabularyRoute() {
  const [words, setWords] = useState<Vocab[]>([]);
  const [reps, setReps] = useState<Replacement[]>([]);
  const [newWord, setNewWord] = useState('');
  const [newReplacement, setNewReplacement] = useState({ from: '', to: '' });

  // v0.6.7 scalability: search filters + explicit sort. Kept separate for
  // each list so one doesn't mask the other. Search is a simple
  // case-insensitive substring match — at the scale we care about
  // (hundreds of entries) anything fancier would be premature. Sort
  // applies only to the words list (replacements already sort by
  // sort_order server-side).
  const [wordsQuery, setWordsQuery] = useState('');
  const [wordsSort, setWordsSort] = useState<WordSort>('alpha');
  const [repsQuery, setRepsQuery] = useState('');

  const reload = async () => {
    const [w, r] = await Promise.all([
      invoke<Vocab[]>('list_vocabulary'),
      invoke<Replacement[]>('list_replacements'),
    ]);
    setWords(w);
    setReps(r);
  };

  useEffect(() => {
    void reload();
  }, []);

  const addWord = async () => {
    if (!newWord.trim()) return;
    await invoke('upsert_vocabulary', {
      entry: { id: '', word: newWord.trim(), hint: null, weight: 1, enabled: true, created_at: 0 },
    });
    setNewWord('');
    await reload();
  };

  const addReplacement = async () => {
    if (!newReplacement.from.trim()) return;
    await invoke('upsert_replacement', {
      r: {
        id: '',
        pattern: newReplacement.from,
        replacement: newReplacement.to,
        enabled: true,
        is_regex: false,
        case_sens: false,
        sort_order: reps.length,
        created_at: 0,
      },
    });
    setNewReplacement({ from: '', to: '' });
    await reload();
  };

  const wordsAtCap = false;
  const repsAtCap = false;

  // Filter + sort pipeline for the visible words list. useMemo so a
  // render caused by something else (e.g. licence status refresh) doesn't
  // resort a long list unnecessarily.
  const visibleWords = useMemo(() => {
    const q = wordsQuery.trim().toLowerCase();
    const filtered = q
      ? words.filter((w) => w.word.toLowerCase().includes(q))
      : words.slice();
    if (wordsSort === 'alpha') {
      filtered.sort((a, b) =>
        a.word.localeCompare(b.word, undefined, { sensitivity: 'base' }),
      );
    }
    // `recent` is the server's natural order (insertion), so no sort.
    return filtered;
  }, [words, wordsQuery, wordsSort]);

  const visibleReps = useMemo(() => {
    const q = repsQuery.trim().toLowerCase();
    if (!q) return reps;
    return reps.filter(
      (r) =>
        r.pattern.toLowerCase().includes(q) ||
        r.replacement.toLowerCase().includes(q),
    );
  }, [reps, repsQuery]);

  // Same "X / N" counter pattern the DESIGN_SYSTEM uses for any bounded
  // count — muted, tabular-nums, sits next to the section title.
  const wordsCounter = `${words.length} word${words.length === 1 ? '' : 's'}`;
  const repsCounter = `${reps.length} replacement${reps.length === 1 ? '' : 's'}`;

  return (
    <div className="mx-auto max-w-4xl p-8">
      <PageHeader
        title="Vocabulary & Replacements"
        subtitle="Teach Utterbar your jargon and shortcuts. Applied locally, never uploaded."
      />

      <QuickAddHint />

      <div className="mt-6 space-y-4">
        <Card
          title="Custom words"
          description="Help the ASR model recognise proper nouns, brand names, or technical terms."
          aside={
            <span className="font-mono text-[11px] tabular-nums text-muted">
              {wordsCounter}
            </span>
          }
        >
          <div className="flex gap-2">
            <Input
              value={newWord}
              onChange={(e) => setNewWord(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && !wordsAtCap && void addWord()}
              placeholder="Add a word (e.g. Kubernetes)"
              className="flex-1"
              disabled={wordsAtCap}
            />
            <Button
              variant="cta"
              size="sm"
              onClick={() => void addWord()}
              disabled={wordsAtCap}
            >
              Add
            </Button>
          </div>
          {/* Search + sort controls. Hidden until the list is worth
              filtering (10+ entries) so small lists don't get chrome they
              don't need. */}
          {words.length >= 10 && (
            <div className="mt-3 flex items-center gap-2">
              <Input
                value={wordsQuery}
                onChange={(e) => setWordsQuery(e.target.value)}
                placeholder={`Search ${words.length} words…`}
                className="flex-1"
              />
              <div className="inline-flex overflow-hidden rounded-[8px] border border-border-soft">
                <button
                  type="button"
                  onClick={() => setWordsSort('alpha')}
                  className={`px-2.5 py-1 text-[11px] transition-colors ${
                    wordsSort === 'alpha'
                      ? 'bg-bg-2 text-fg-hi'
                      : 'text-muted hover:text-fg-hi'
                  }`}
                  title="Sort A–Z"
                >
                  A–Z
                </button>
                <button
                  type="button"
                  onClick={() => setWordsSort('recent')}
                  className={`px-2.5 py-1 text-[11px] transition-colors ${
                    wordsSort === 'recent'
                      ? 'bg-bg-2 text-fg-hi'
                      : 'text-muted hover:text-fg-hi'
                  }`}
                  title="Sort by most recently added"
                >
                  Recent
                </button>
              </div>
            </div>
          )}
          {/* Tightened row density (py-1.5 rather than py-2) so more of a
              long list fits on-screen at once. Row height is still tall
              enough for a comfortable click target. */}
          <ul className="mt-3 divide-y divide-border-soft/50">
            {visibleWords.map((w) => (
              <li
                key={w.id}
                className="flex items-center justify-between py-1.5 text-[13px] text-fg"
              >
                <span className="truncate">{w.word}</span>
                <button
                  onClick={() => invoke('delete_vocabulary', { id: w.id }).then(reload)}
                  className="shrink-0 text-[11px] text-fg-dim transition-colors hover:text-rose"
                >
                  Remove
                </button>
              </li>
            ))}
            {words.length === 0 && (
              <li className="py-3 text-[12px] text-muted">No words yet.</li>
            )}
            {words.length > 0 && visibleWords.length === 0 && (
              <li className="py-3 text-[12px] text-muted">
                No matches for “{wordsQuery}”.
              </li>
            )}
          </ul>
        </Card>

        <Card
          title="Text replacements"
          description="Swap literal phrases post-ASR. Useful for common typos or shorthand expansion."
          aside={
            <span className="font-mono text-[11px] tabular-nums text-muted">
              {repsCounter}
            </span>
          }
        >
          <div className="flex gap-2">
            <Input
              value={newReplacement.from}
              onChange={(e) => setNewReplacement((r) => ({ ...r, from: e.target.value }))}
              placeholder="Match"
              className="flex-1"
              disabled={repsAtCap}
            />
            <Input
              value={newReplacement.to}
              onChange={(e) => setNewReplacement((r) => ({ ...r, to: e.target.value }))}
              onKeyDown={(e) => e.key === 'Enter' && !repsAtCap && void addReplacement()}
              placeholder="Replace with"
              className="flex-1"
              disabled={repsAtCap}
            />
            <Button
              variant="cta"
              size="sm"
              onClick={() => void addReplacement()}
              disabled={repsAtCap}
            >
              Add
            </Button>
          </div>
          {reps.length >= 10 && (
            <div className="mt-3">
              <Input
                value={repsQuery}
                onChange={(e) => setRepsQuery(e.target.value)}
                placeholder={`Search ${reps.length} replacements…`}
              />
            </div>
          )}
          <ul className="mt-3 divide-y divide-border-soft/50">
            {visibleReps.map((r) => (
              <li
                key={r.id}
                className="flex items-center justify-between py-1.5 text-[13px] text-fg"
              >
                <span className="min-w-0 truncate">
                  <span className="font-mono text-[12px]">{r.pattern}</span>
                  <span className="mx-2 text-fg-dim">→</span>
                  <span className="font-mono text-[12px]">{r.replacement}</span>
                </span>
                <button
                  onClick={() => invoke('delete_replacement', { id: r.id }).then(reload)}
                  className="shrink-0 text-[11px] text-fg-dim transition-colors hover:text-rose"
                >
                  Remove
                </button>
              </li>
            ))}
            {reps.length === 0 && (
              <li className="py-3 text-[12px] text-muted">No replacements yet.</li>
            )}
            {reps.length > 0 && visibleReps.length === 0 && (
              <li className="py-3 text-[12px] text-muted">
                No matches for “{repsQuery}”.
              </li>
            )}
          </ul>
        </Card>

        <Card title="Guardrails">
          <Row label="Warn when vocabulary list exceeds 500 entries">
            <span className="text-[12px] text-muted">Recommended on</span>
          </Row>
        </Card>
      </div>
    </div>
  );
}

/**
 * Quick-add hint sitting under the Vocabulary page header.
 *
 * Two states:
 *   - Shortcut bound: shows the bound accelerator + a "Capture selection
 *     now" button that triggers the same workflow without leaving the page.
 *   - Shortcut unbound: shows a "Bind a shortcut" CTA that jumps to
 *     Settings → Shortcuts. This is the discoverability surface for the
 *     feature — the page where users actually look for vocabulary tools.
 *
 * The capture button works regardless of binding because it goes through
 * the same `open_vocab_quick_add` command as the global shortcut. We
 * keep it visible in both states so a Pro user without a shortcut bound
 * still has a one-click path to the quick-add window.
 *
 * The card is intentionally compact — it's a hint, not a feature panel.
 * The full feature lives in the global shortcut + the small quick-add
 * window. Anything more here would crowd the page.
 */
function QuickAddHint() {
  const navigate = useNavigate();
  const shortcut = useSettings(
    (s) =>
      (s.data?.shortcuts as { add_to_vocabulary?: { label?: string } } | undefined)
        ?.add_to_vocabulary,
  );
  const label = shortcut?.label;

  const captureNow = async () => {
    try {
      await invoke('open_vocab_quick_add');
    } catch {
      // The Rust side already swallows transport errors and opens an
      // empty window; if the invoke itself fails the user has bigger
      // problems than a missing toast.
    }
  };

  return (
    <div className="mt-5 flex items-center justify-between gap-3 rounded-card bg-bg-1 px-4 py-3">
      <div className="flex items-start gap-2.5">
        <Zap className="mt-0.5 h-3.5 w-3.5 shrink-0 text-accent" />
        <div className="text-[12px] leading-relaxed text-fg">
          <span className="font-medium text-fg-hi">Add words from any app.</span>{' '}
          Select a word, press your add-to-vocabulary shortcut, and confirm. Utterbar
          synthesises Ctrl+C and reads the clipboard, then puts it back.
          <span className="block text-[11px] text-muted">
            Plain-text clipboard only — image, file or rich-text clipboard contents
            are lost during capture.
          </span>
        </div>
      </div>
      <div className="flex shrink-0 items-center gap-2">
        {label ? (
          <Kbd>{label}</Kbd>
        ) : (
          <Button
            variant="ghost"
            size="sm"
            noLed
            onClick={() => navigate('/settings/shortcuts')}
          >
            Bind a shortcut →
          </Button>
        )}
        <Button variant="ghost" size="sm" noLed onClick={() => void captureNow()}>
          <Plus className="h-3 w-3" />
          Capture
        </Button>
      </div>
    </div>
  );
}
