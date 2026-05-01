import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { useSettings } from '@/state/settings';

export function TextOutputSettings() {
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  if (!s) return null;

  const update = (key: string, value: unknown) =>
    patch({ text_output: { ...s.text_output, [key]: value } });

  const t = s.text_output as Record<string, boolean>;

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader title="Text Output" subtitle="How dictated text reaches your active app." />
      <div className="mt-6 space-y-4">
        <Card title="Delivery">
          <Row
            label="Paste automatically"
            description="After transcription, copy the result to the clipboard and simulate Ctrl+V. This is the fastest, most reliable path, most apps accept it."
          >
            <Toggle checked={t.paste_automatically} onChange={(v) => void update('paste_automatically', v)} />
          </Row>
          <Row
            label="Restore previous clipboard"
            description="About 1.5 seconds after pasting, put whatever was on your clipboard before back. Keeps your copy buffer intact."
          >
            <Toggle checked={t.restore_clipboard} onChange={(v) => void update('restore_clipboard', v)} />
          </Row>
          <Row
            label="Type it out if paste is blocked"
            description="Some apps (admin tools, remote desktop, a few games) don't accept pasted text. When that happens, Utterbar types the words in one character at a time instead. Slower, but it reaches apps that would otherwise drop the paste. Leave this on unless you've got a reason not to."
          >
            <Toggle checked={t.keystroke_fallback} onChange={(v) => void update('keystroke_fallback', v)} />
          </Row>
        </Card>

        <Card title="Formatting">
          <Row
            label="Auto-capitalise first letter"
            description="Make sure the inserted text starts with a capital letter, regardless of how the model transcribed it."
          >
            <Toggle checked={t.auto_capitalise} onChange={(v) => void update('auto_capitalise', v)} />
          </Row>
          <Row
            label="Trim trailing whitespace"
            description="Remove any spaces, tabs, or newlines at the end of the transcribed text before it gets inserted."
          >
            <Toggle checked={t.trim_trailing_space} onChange={(v) => void update('trim_trailing_space', v)} />
          </Row>
          <Row
            label="Auto-space between dictations"
            description="When you dictate two things back-to-back in the same app, add a space between them if the first one didn't already end with whitespace. Keeps 'hello' + 'world' from colliding into 'helloworld'."
          >
            <Toggle
              checked={t.auto_space_on_paste ?? true}
              onChange={(v) => void update('auto_space_on_paste', v)}
            />
          </Row>
          <Row
            label="Numbers as digits"
            description="Convert spoken numbers into digits (&lsquo;twenty three&rsquo; &rarr; &lsquo;23&rsquo;, &lsquo;five pounds&rsquo; &rarr; &lsquo;5 pounds&rsquo;). Idioms stay as words (&lsquo;one of these days&rsquo;). Turn off if you prefer numbers spelled out."
          >
            <Toggle
              checked={t.numbers_as_digits ?? true}
              onChange={(v) => void update('numbers_as_digits', v)}
            />
          </Row>
          <Row
            label="Smart rewrite"
            description="Clean up fillers (&lsquo;um&rsquo;, &lsquo;uh&rsquo;, &lsquo;like&rsquo;), spacing, capitalisation, and obvious punctuation locally before insertion. Deterministic only &mdash; no AI model, no cloud service, no model download. Adds a small amount of processing time. Off by default. Meeting Mode is unaffected (transcripts stay verbatim)."
          >
            <Toggle
              checked={t.smart_rewrite ?? false}
              onChange={(v) => void update('smart_rewrite', v)}
            />
          </Row>
          <Row
            label="Hold Shift to auto-send"
            description="If you're holding Shift when you stop recording, Utterbar presses Enter for you right after pasting, so the message goes out in Teams, Slack, Discord, etc. without a second hand-off."
          >
            <Toggle checked={t.shift_to_auto_send} onChange={(v) => void update('shift_to_auto_send', v)} />
          </Row>
        </Card>
      </div>
    </div>
  );
}

