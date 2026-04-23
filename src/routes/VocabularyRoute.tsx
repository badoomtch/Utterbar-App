import { useEffect, useState } from 'react';

import { Button } from '@/components/common/Button';
import { Card, Row } from '@/components/common/Card';
import { Input } from '@/components/common/Input';
import { PageHeader } from '@/components/common/PageHeader';
import { invoke } from '@/lib/ipc';

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
        >
          <div className="flex gap-2">
            <Input
              value={newWord}
              onChange={(e) => setNewWord(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && void addWord()}
              placeholder="Add a word (e.g. Kubernetes)"
              className="flex-1"
            />
            <Button variant="cta" size="sm" onClick={() => void addWord()}>
              Add
            </Button>
          </div>
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
        >
          <div className="flex gap-2">
            <Input
              value={newReplacement.from}
              onChange={(e) => setNewReplacement((r) => ({ ...r, from: e.target.value }))}
              placeholder="Match"
              className="flex-1"
            />
            <Input
              value={newReplacement.to}
              onChange={(e) => setNewReplacement((r) => ({ ...r, to: e.target.value }))}
              onKeyDown={(e) => e.key === 'Enter' && void addReplacement()}
              placeholder="Replace with"
              className="flex-1"
            />
            <Button variant="cta" size="sm" onClick={() => void addReplacement()}>
              Add
            </Button>
          </div>
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
