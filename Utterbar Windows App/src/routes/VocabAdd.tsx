import { useEffect, useRef, useState } from 'react';
import { Plus, X } from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { Button } from '@/components/common/Button';
import { Input } from '@/components/common/Input';
import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import type { VocabCapturePayload } from '@/generated/VocabCapturePayload';

/**
 * Quick add-to-vocabulary window.
 *
 * Opened by the global "Add to vocabulary" shortcut (Settings → Shortcuts)
 * or by the Vocabulary page's "Capture selection" button. The Rust side
 * synthesises Ctrl+C against the foreground app, reads the resulting
 * clipboard, restores the previous plain-text clipboard, and emits
 * `vocab:capture` with the sanitised text.
 *
 * What this window does NOT do:
 *   - It does NOT preserve image / file / RTF clipboard content. If the
 *     user had a screenshot in their clipboard before invoking the
 *     workflow, that content is lost (only plain text is restored). This
 *     is documented behaviour — see the trade-off note in
 *     `commands/vocabulary.rs` and the Vocabulary page.
 *   - It does NOT capture long selections. Anything past 60 chars or 5
 *     words is rejected up front; the user gets an explanation and an
 *     empty input rather than a junk auto-fill.
 *
 * The window is small (360×220), always-on-top, centred on the active
 * monitor. One purpose: type or confirm a word and hit Enter to save.
 * Esc / X closes without saving.
 */
export function VocabAdd() {
  const [word, setWord] = useState('');
  const [hint, setHint] = useState<'idle' | 'rejected' | 'empty-capture'>('empty-capture');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [savedFlash, setSavedFlash] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // Listen for capture payloads from the Rust side. The window is opened
  // before the emit fires (commands/vocabulary.rs::capture_and_open) so
  // the listener is guaranteed to be mounted. We accept the empty payload
  // case — that's how Rust signals "user had nothing selected".
  useEffect(() => {
    const off = listen<VocabCapturePayload>(EVT.vocabCapture, (e) => {
      const p = e.payload;
      if (p.rejected_too_long) {
        setHint('rejected');
        setWord('');
      } else if (p.captured) {
        setHint('idle');
        setWord(p.captured);
      } else {
        setHint('empty-capture');
        setWord('');
      }
      // Pull focus into the input so the user can immediately edit or
      // hit Enter. setTimeout pushes this after the state flush.
      setTimeout(() => inputRef.current?.focus(), 0);
    });
    return () => {
      void off.then((u) => u()).catch(() => undefined);
    };
  }, []);

  // Esc closes; Enter saves.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') void close();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  const close = async () => {
    try {
      await getCurrentWindow().close();
    } catch {
      // Some webview shutdown races throw — ignoring is safe; the OS
      // will close the window when the process tears down.
    }
  };

  const save = async () => {
    const trimmed = word.trim();
    if (!trimmed) return;
    setSaving(true);
    setError(null);
    try {
      await invoke('upsert_vocabulary', {
        entry: {
          id: '',
          word: trimmed,
          hint: null,
          weight: 1,
          enabled: true,
          created_at: 0,
        },
      });
      setSavedFlash(true);
      // Brief confirmation flash, then close. Keeps the workflow under
      // half a second from shortcut press to back-in-app.
      setTimeout(() => void close(), 320);
    } catch (e) {
      setSaving(false);
      const msg =
        (e as { message?: string } | null)?.message ?? 'Could not save. Try again.';
      // Free-tier cap message is the expected error here. Surface it
      // verbatim — the Rust side already produces user-readable copy
      // ("Vocabulary cap reached (50 entries on Free)").
      setError(typeof msg === 'string' ? msg : 'Could not save. Try again.');
    }
  };

  const onSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    void save();
  };

  return (
    <div className="flex h-full min-h-0 w-full flex-col bg-bg-1 text-fg">
      <header className="flex items-center justify-between px-4 pt-3">
        <div className="flex items-center gap-2">
          <Plus className="h-3.5 w-3.5 text-accent" />
          <span className="text-[12px] font-medium text-fg-hi">
            Add to vocabulary
          </span>
        </div>
        <button
          onClick={() => void close()}
          aria-label="Close"
          className="text-muted transition-colors hover:text-fg-hi"
        >
          <X className="h-3.5 w-3.5" />
        </button>
      </header>

      <form onSubmit={onSubmit} className="flex flex-1 flex-col px-4 py-3">
        <Input
          ref={inputRef}
          value={word}
          onChange={(e) => setWord(e.target.value)}
          placeholder="Word or short phrase…"
          autoFocus
          maxLength={60}
        />
        <div className="mt-2 min-h-[32px] text-[11px] leading-snug text-muted">
          {hint === 'rejected'
            ? "That selection was too long for a vocabulary entry. Pick a shorter word or short phrase."
            : hint === 'empty-capture' && !word
              ? 'Select a word in any app, then trigger the shortcut. Or type one here.'
              : 'Press Enter to save. Esc to close.'}
        </div>
        {error && (
          <div className="mt-1 text-[11px] text-[color:var(--danger,#f87171)]">{error}</div>
        )}
        <div className="mt-auto flex items-center justify-end gap-2 pt-2">
          <button
            type="button"
            onClick={() => void close()}
            className="text-[12px] text-muted transition-colors hover:text-fg-hi"
          >
            Cancel
          </button>
          <Button
            type="submit"
            variant="cta"
            size="sm"
            disabled={saving || !word.trim()}
          >
            {savedFlash ? 'Saved' : saving ? 'Saving…' : 'Add to vocabulary'}
          </Button>
        </div>
      </form>
    </div>
  );
}
