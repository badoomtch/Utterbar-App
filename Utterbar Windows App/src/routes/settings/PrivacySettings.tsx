import { useEffect, useState } from 'react';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { ProNudge } from '@/components/common/ProNudge';
import { Toggle } from '@/components/common/Toggle';
import { invoke } from '@/lib/ipc';
import { useLicence, isPro } from '@/state/licence';
import { useSettings } from '@/state/settings';

type DataPaths = {
  config: string;
  data: string;
  models: string;
  recordings: string;
  logs: string;
};

export function PrivacySettings() {
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  const licenceStatus = useLicence((x) => x.status);
  const [paths, setPaths] = useState<DataPaths | null>(null);

  useEffect(() => {
    void invoke<DataPaths>('get_data_paths').then(setPaths).catch(() => setPaths(null));
  }, []);

  if (!s) return null;
  const pro = isPro(licenceStatus);
  // PrivacySettings is typed end-to-end via `@/generated/PrivacySettings`, but this
  // screen intentionally drives every field through a generic `update(key, value)`
  // helper so the JSX stays flat. Cast once here to a loose index so the per-field
  // reads/writes don't each need their own narrowing. Fields that aren't scalar
  // (conferencing_apps_override: string[]) are never touched through `p`, they go
  // through `s.privacy.*` directly below.
  const p = s.privacy as unknown as Record<string, boolean | number>;

  const update = (key: string, value: unknown) =>
    patch({ privacy: { ...s.privacy, [key]: value } });

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader
        title="Privacy"
        subtitle="Utterbar processes everything on-device. You are always in control of what is retained and what is used as context."
      />

      <div className="mt-6 space-y-4">
        <Card title="Context capture" description="Which signals may a mode use to tailor its output?">
          <Row label="Allow active-app context" description="App executable and window title.">
            <Toggle
              checked={Boolean(p.allow_context_active_app)}
              onChange={(v) => void update('allow_context_active_app', v)}
            />
          </Row>
          <Row label="Allow selection context" description="Currently selected text in the foreground app.">
            <Toggle
              checked={Boolean(p.allow_context_selection)}
              onChange={(v) => void update('allow_context_selection', v)}
            />
          </Row>
          <Row label="Allow clipboard context" description="Most recently copied text.">
            <Toggle
              checked={Boolean(p.allow_context_clipboard)}
              onChange={(v) => void update('allow_context_clipboard', v)}
            />
          </Row>
        </Card>

        <Card title="Audio retention">
          <Row
            label="Save recordings"
            description="Keep the WAV of each dictation on disk so you can re-play or re-transcribe it later. Files live under your Recordings folder and are automatically pruned after the retention window below."
          >
            <Toggle
              checked={pro && Boolean(p.retain_audio)}
              onChange={(v) => void update('retain_audio', v)}
              disabled={!pro}
            />
          </Row>
          {!pro && (
            <ProNudge
              feature="Audio retention"
              description="Keep a local WAV of every dictation so you can re-listen or re-transcribe it later. Stored on your disk only, never uploaded."
              dense
              className="mt-1"
            />
          )}
          <Row
            label="Delete recordings after"
            description="Older WAVs are cleaned up automatically on each new dictation, so the recordings folder doesn't balloon."
          >
            <div className="flex items-center gap-3">
              <input
                type="range"
                min={1}
                max={90}
                step={1}
                disabled={!pro || !p.retain_audio}
                value={Number(p.retain_audio_days ?? 14)}
                onChange={(e) => void update('retain_audio_days', Number(e.target.value))}
                className="w-40"
              />
              <span className="w-16 text-right font-mono text-[12px] tabular-nums text-fg-hi">
                {Number(p.retain_audio_days ?? 14)} days
              </span>
            </div>
          </Row>
        </Card>

        <Card title="Media">
          <Row
            label="Pause music while recording"
            description="Automatically pause Spotify, YouTube, Apple Music, and any other app registered with Windows media controls while you dictate. Only the sessions Utterbar paused get resumed."
          >
            <Toggle
              checked={Boolean(p.pause_media_while_recording)}
              onChange={(v) => void update('pause_media_while_recording', v)}
            />
          </Row>
          <Row
            label="Mute conferencing apps while recording"
            description="Silences Teams, Slack, Discord, Zoom, and Webex output audio while you're dictating, so call audio doesn't bleed into the room. Restored the moment dictation ends. Doesn't mute your mic to those apps, use the app's own mute button for that."
          >
            <Toggle
              checked={Boolean(p.mute_conferencing_while_recording)}
              onChange={(v) => void update('mute_conferencing_while_recording', v)}
            />
          </Row>
        </Card>

        <Card title="Data locations">
          {paths &&
            (
              [
                ['Config', paths.config],
                ['Database', paths.data],
                ['Models', paths.models],
                ['Recordings', paths.recordings],
                ['Logs', paths.logs],
              ] as const
            ).map(([k, v]) => (
              <Row key={k} label={k}>
                <code className="font-mono text-[11px] text-muted">{v}</code>
              </Row>
            ))}
        </Card>

      </div>
    </div>
  );
}
