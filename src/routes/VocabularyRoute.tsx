import { useEffect, useState } from 'react';

import { Button } from '@/components/common/Button';
import { Card, Row } from '@/components/common/Card';
import { Input } from '@/components/common/Input';
import { PageHeader } from '@/components/common/PageHeader';
import { ProNudge } from '@/components/common/ProNudge';
import { invoke } from '@/lib/ipc';
import { useLicence, isPro } from '@/state/licence';

// Must track the backend caps in src-tauri/src/util/gating.rs. If you edit
// one, edit the other — there's no shared source of truth for "50 / 25"
// that bridges Rust and TS (these are display values, not validation —
// the backend is the enforcement).
const FREE_VOCAB_CAP = 50;
const FREE_REPLACEMENTS_CAP = 25;

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
export function VocabularyRoute() {
  const [words, setWords] = useState<Vocab[]>([]);
  const [reps, setReps] = useState<Replacement[]>([]);
  const [newWord, setNewWord] = useState('');
  const [newReplacement, setNewReplacement] = useState({ from: '', to: '' });

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

  const licenceStatus = useLicence((s) => s.status);
  const pro = isPro(licenceStatus);
  const wordsAtCap = !pro && words.length >= FREE_VOCAB_CAP;
  const repsAtCap = !pro && reps.length >= FREE_REPLACEMENTS_CAP;

  // Same "X / N" counter pattern the DESIGN_SYSTEM uses for any bounded
  // count — muted, tabular-nums, sits next to the section title.
  const wordsCounter = pro
    ? `${words.length} word${words.length === 1 ? '' : 's'}`
    : `${words.length} / ${FREE_VOCAB_CAP} words`;
  const repsCounter = pro
    ? `${reps.length} replacement${reps.length === 1 ? '' : 's'}`
    : `${reps.length} / ${FREE_REPLACEMENTS_CAP} replacements`;

  return (
    <div className="mx-auto max-w-4xl p-8">
      <PageHeader
        title="Vocabulary & Replacements"
        subtitle="Teach Utterbar your jargon and shortcuts. Applied locally, never uploaded."
      />

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
          {wordsAtCap && (
            <ProNudge
              feature="Unlimited vocabulary"
              description={`You're at the Free-tier cap of ${FREE_VOCAB_CAP} words. Existing words keep working — Pro removes the cap.`}
              dense
              className="mt-3"
            />
          )}
          <ul className="mt-3 divide-y divide-border-soft/50">
            {words.map((w) => (
              <li
                key={w.id}
                className="flex items-center justify-between py-2 text-[13px] text-fg"
              >
                <span>{w.word}</span>
                <button
                  onClick={() => invoke('delete_vocabulary', { id: w.id }).then(reload)}
                  className="text-[11px] text-fg-dim transition-colors hover:text-rose"
                >
                  Remove
                </button>
              </li>
            ))}
            {words.length === 0 && (
              <li className="py-3 text-[12px] text-muted">No words yet.</li>
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
          {repsAtCap && (
            <ProNudge
              feature="Unlimited replacements"
              description={`You're at the Free-tier cap of ${FREE_REPLACEMENTS_CAP} replacements. Existing rules keep working — Pro removes the cap.`}
              dense
              className="mt-3"
            />
          )}
          <ul className="mt-3 divide-y divide-border-soft/50">
            {reps.map((r) => (
              <li
                key={r.id}
                className="flex items-center justify-between py-2 text-[13px] text-fg"
              >
                <span>
                  <span className="font-mono text-[12px]">{r.pattern}</span>
                  <span className="mx-2 text-fg-dim">→</span>
                  <span className="font-mono text-[12px]">{r.replacement}</span>
                </span>
                <button
                  onClick={() => invoke('delete_replacement', { id: r.id }).then(reload)}
                  className="text-[11px] text-fg-dim transition-colors hover:text-rose"
                >
                  Remove
                </button>
              </li>
            ))}
            {reps.length === 0 && (
              <li className="py-3 text-[12px] text-muted">No replacements yet.</li>
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
