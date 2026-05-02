import { useEffect, useMemo, useState } from 'react';

import { PageHeader } from '@/components/common/PageHeader';
import { Button } from '@/components/common/Button';
import { Card, Row } from '@/components/common/Card';
import { Input } from '@/components/common/Input';
import { ManagedHint } from '@/components/common/ManagedHint';
import { ProNudge } from '@/components/common/ProNudge';
import { Toggle } from '@/components/common/Toggle';
import { invoke } from '@/lib/ipc';
import { useLicence, isPro } from '@/state/licence';
import { usePolicy } from '@/state/policy';
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
  const policy = usePolicy((x) => x.snapshot);
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

      {/* Managed-deployment hint. Renders only when at least one of
          the privacy-adjacent group policies is set (history /
          export / telemetry). The cards below stay editable for
          user-controlled fields; this banner is the page-level
          explanation for anything the user finds disabled. */}
      {policy &&
        (policy.history_blocked ||
          policy.history_export_blocked ||
          policy.telemetry_blocked) && (
          <div className="mt-6">
            <ManagedHint variant="block">
              Some privacy controls on this device are managed by your administrator.
              Restart Utterbar to pick up policy changes.
            </ManagedHint>
          </div>
        )}

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
            label="Mute conferencing app output while recording"
            description="Silences what you HEAR from Teams, Slack, Discord, Zoom, and Webex while you're dictating, so call audio and notification dings don't bleed into the room. Restored the moment dictation ends. This does NOT mute your microphone inside those apps, use each app's own mute button for that."
          >
            <Toggle
              checked={Boolean(p.mute_conferencing_while_recording)}
              onChange={(v) => void update('mute_conferencing_while_recording', v)}
            />
          </Row>
          <ConferencingAppsEditor
            override={s.privacy.conferencing_apps_override}
            disabled={!p.mute_conferencing_while_recording}
            onChange={(next) => void update('conferencing_apps_override', next)}
          />
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

// Maps the stable reason strings returned by the `validate_conferencing_app_name`
// Tauri command to the user-facing message rendered next to the input. Keeping
// this keyed off the same constants the Rust validator returns means the copy
// stays in lock-step with the rule set without re-implementing validation in TS.
const VALIDATION_MESSAGES: Record<string, string> = {
  empty: 'Enter an executable name.',
  invalid_characters:
    'Only letters, numbers, dot, underscore, and dash. No paths, drive letters, quotes, or wildcards.',
  too_long: 'Name is too long (max 64 characters).',
  duplicate: 'That app is already in the list.',
};

function ConferencingAppsEditor({
  override,
  disabled,
  onChange,
}: {
  override: string[];
  disabled: boolean;
  onChange: (next: string[]) => void;
}) {
  const [defaults, setDefaults] = useState<string[]>([]);
  const [draft, setDraft] = useState('');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void invoke<string[]>('get_default_conferencing_apps')
      .then(setDefaults)
      .catch(() => setDefaults([]));
  }, []);

  // Empty override means "use the built-in defaults" (matches the runtime
  // contract in services/audio_ducking.rs). The editor renders the effective
  // list either way so users can see what will actually be muted.
  const usingDefaults = override.length === 0;
  const effective = useMemo(
    () => (usingDefaults ? defaults : override),
    [usingDefaults, defaults, override],
  );

  const add = async () => {
    setError(null);
    try {
      const canonical = await invoke<string>('validate_conferencing_app_name', {
        name: draft,
        existing: effective,
      });
      // First user edit forks the list off the defaults — seed the override
      // with the current effective list so the user sees the same set they
      // were just looking at, plus the new entry.
      const base = usingDefaults ? defaults : override;
      onChange([...base, canonical]);
      setDraft('');
    } catch (e) {
      const reason = typeof e === 'string' ? e : 'invalid_characters';
      setError(VALIDATION_MESSAGES[reason] ?? 'That name isn’t valid.');
    }
  };

  const remove = (name: string) => {
    setError(null);
    // If we’re still on defaults, removing an item forks the list (seed
    // with the remaining defaults). Otherwise drop it from the override.
    const base = usingDefaults ? defaults : override;
    const next = base.filter((n) => n.toLowerCase() !== name.toLowerCase());
    onChange(next);
  };

  const reset = () => {
    setError(null);
    onChange([]);
  };

  return (
    <div
      className={
        'border-t border-border-soft/70 pt-4 ' + (disabled ? 'opacity-60' : '')
      }
    >
      <div className="flex items-baseline justify-between gap-3">
        <div>
          <div className="text-[13px] font-medium text-fg-hi">Apps to silence</div>
          <div className="mt-1 text-[12px] leading-relaxed text-muted">
            Executable names matched case-insensitively against running processes.
            {usingDefaults
              ? ' Using the built-in defaults.'
              : ' Using your custom list.'}
          </div>
        </div>
        {!usingDefaults && (
          <Button
            variant="plain"
            size="sm"
            onClick={reset}
            disabled={disabled}
          >
            Reset to defaults
          </Button>
        )}
      </div>

      <div className="mt-3 flex flex-wrap gap-1.5">
        {effective.length === 0 ? (
          <span className="text-[12px] text-muted">No apps configured.</span>
        ) : (
          effective.map((name) => (
            <span
              key={name}
              className="inline-flex items-center gap-1 rounded-full bg-bg-0 px-2.5 py-1 font-mono text-[11px] text-fg-hi"
            >
              {name}
              <button
                type="button"
                onClick={() => remove(name)}
                disabled={disabled}
                aria-label={`Remove ${name}`}
                className="ml-0.5 text-muted hover:text-fg-hi disabled:cursor-not-allowed disabled:opacity-50"
              >
                &times;
              </button>
            </span>
          ))
        )}
      </div>

      <form
        className="mt-3 flex items-center gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          if (!disabled && draft.trim()) void add();
        }}
      >
        <Input
          value={draft}
          onChange={(e) => {
            setDraft(e.target.value);
            if (error) setError(null);
          }}
          placeholder="e.g. Teams.exe"
          disabled={disabled}
          className="flex-1"
        />
        <Button
          variant="ghost"
          size="sm"
          type="submit"
          disabled={disabled || !draft.trim()}
        >
          Add
        </Button>
      </form>
      {error && (
        <div className="mt-2 text-[12px] text-rose-400" role="alert">
          {error}
        </div>
      )}
    </div>
  );
}
