import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

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
  const { t } = useTranslation('settings');
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
      <PageHeader title={t('privacy.title')} subtitle={t('privacy.subtitle')} />

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
            <ManagedHint variant="block">{t('privacy.managed_hint')}</ManagedHint>
          </div>
        )}

      <div className="mt-6 space-y-4">
        <Card
          title={t('privacy.context.title')}
          description={t('privacy.context.description')}
        >
          <Row
            label={t('privacy.context.active_app.label')}
            description={t('privacy.context.active_app.description')}
          >
            <Toggle
              checked={Boolean(p.allow_context_active_app)}
              onChange={(v) => void update('allow_context_active_app', v)}
            />
          </Row>
          <Row
            label={t('privacy.context.selection.label')}
            description={t('privacy.context.selection.description')}
          >
            <Toggle
              checked={Boolean(p.allow_context_selection)}
              onChange={(v) => void update('allow_context_selection', v)}
            />
          </Row>
          <Row
            label={t('privacy.context.clipboard.label')}
            description={t('privacy.context.clipboard.description')}
          >
            <Toggle
              checked={Boolean(p.allow_context_clipboard)}
              onChange={(v) => void update('allow_context_clipboard', v)}
            />
          </Row>
        </Card>

        <Card title={t('privacy.audio_retention.title')}>
          <Row
            label={t('privacy.audio_retention.save_recordings.label')}
            description={t('privacy.audio_retention.save_recordings.description')}
          >
            <Toggle
              checked={pro && Boolean(p.retain_audio)}
              onChange={(v) => void update('retain_audio', v)}
              disabled={!pro}
            />
          </Row>
          {!pro && (
            <ProNudge
              feature={t('privacy.audio_retention.pro_nudge.feature')}
              description={t('privacy.audio_retention.pro_nudge.description')}
              dense
              className="mt-1"
            />
          )}
          <Row
            label={t('privacy.audio_retention.delete_after.label')}
            description={t('privacy.audio_retention.delete_after.description')}
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
                {t('privacy.audio_retention.delete_after.value', {
                  days: Number(p.retain_audio_days ?? 14),
                })}
              </span>
            </div>
          </Row>
        </Card>

        <Card title={t('privacy.media.title')}>
          <Row
            label={t('privacy.media.pause_music.label')}
            description={t('privacy.media.pause_music.description')}
          >
            <Toggle
              checked={Boolean(p.pause_media_while_recording)}
              onChange={(v) => void update('pause_media_while_recording', v)}
            />
          </Row>
          <Row
            label={t('privacy.media.mute_conferencing.label')}
            description={t('privacy.media.mute_conferencing.description')}
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

        <Card title={t('privacy.data_locations.title')}>
          {paths &&
            // Each row's value (paths.config, paths.data, paths.models,
            // paths.recordings, paths.logs) is a backend-emitted
            // filesystem path and stays verbatim. Only the row label is
            // catalogued, keyed off a stable id that maps to the same
            // DataPaths field.
            (
              [
                ['config', paths.config],
                ['database', paths.data],
                ['models', paths.models],
                ['recordings', paths.recordings],
                ['logs', paths.logs],
              ] as const
            ).map(([k, v]) => (
              <Row key={k} label={t(`privacy.data_locations.rows.${k}` as const)}>
                <code className="font-mono text-[11px] text-muted">{v}</code>
              </Row>
            ))}
        </Card>

      </div>
    </div>
  );
}

// The reason ids returned by `validate_conferencing_app_name` (Rust)
// stay as the contract — the matching user-facing message lives in the
// catalogue under privacy.conferencing_apps.errors.<id>. Pin the exact
// reason ids the Rust validator emits so a future regression that
// renames a reason on either side fails loudly at translation time
// instead of silently falling through to the fallback message.
const VALIDATION_REASON_IDS = ['empty', 'invalid_characters', 'too_long', 'duplicate'] as const;
type ValidationReasonId = (typeof VALIDATION_REASON_IDS)[number];

function ConferencingAppsEditor({
  override,
  disabled,
  onChange,
}: {
  override: string[];
  disabled: boolean;
  onChange: (next: string[]) => void;
}) {
  const { t } = useTranslation('settings');
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
    // Guard: if the defaults fetch hasn't landed yet and we'd otherwise
    // fork the list with an empty base, fail soft — pressing Add a moment
    // later will work. Without this, a fast first-add on a slow machine
    // would silently drop the entire default set into a one-entry override.
    if (usingDefaults && defaults.length === 0) {
      setError(t('privacy.conferencing_apps.errors.defaults_loading'));
      return;
    }
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
      // Original semantics: if Rust returned a non-string error, treat
      // it as `invalid_characters` (the most common cause); if Rust
      // returned a string reason we don't have a known message for,
      // show the generic fallback. Catalogue keys mirror the original
      // VALIDATION_MESSAGES dictionary one-for-one plus the fallback.
      if (typeof e !== 'string') {
        setError(t('privacy.conferencing_apps.errors.invalid_characters'));
      } else if ((VALIDATION_REASON_IDS as readonly string[]).includes(e)) {
        setError(t(`privacy.conferencing_apps.errors.${e as ValidationReasonId}` as const));
      } else {
        setError(t('privacy.conferencing_apps.errors.fallback'));
      }
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
      aria-disabled={disabled}
    >
      <div className="flex items-baseline justify-between gap-3">
        <div>
          <div className="text-[13px] font-medium text-fg-hi">
            {t('privacy.conferencing_apps.title')}
          </div>
          <div className="mt-1 text-[12px] leading-relaxed text-muted">
            {t('privacy.conferencing_apps.description_prefix')}{' '}
            {usingDefaults
              ? t('privacy.conferencing_apps.using_defaults')
              : t('privacy.conferencing_apps.using_custom')}
            {disabled && ' ' + t('privacy.conferencing_apps.edit_locked')}
          </div>
        </div>
        {!usingDefaults && (
          <Button
            variant="plain"
            size="sm"
            onClick={reset}
            disabled={disabled}
          >
            {t('privacy.conferencing_apps.reset_to_defaults')}
          </Button>
        )}
      </div>

      <div className="mt-3 flex flex-wrap gap-1.5">
        {effective.length === 0 ? (
          <span className="text-[12px] text-muted">
            {t('privacy.conferencing_apps.empty')}
          </span>
        ) : (
          // Each `name` is a backend-/user-supplied executable name
          // (e.g. "Teams.exe") and is rendered verbatim. Only the
          // surrounding chrome is catalogued.
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
                aria-label={t('privacy.conferencing_apps.remove_aria', { name })}
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
          placeholder={t('privacy.conferencing_apps.input_placeholder')}
          disabled={disabled}
          className="flex-1"
        />
        <Button
          variant="ghost"
          size="sm"
          type="submit"
          disabled={disabled || !draft.trim()}
        >
          {t('privacy.conferencing_apps.add')}
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
