import { useEffect, useState } from 'react';
import { Mail } from 'lucide-react';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { Input } from '@/components/common/Input';
import { useSettings } from '@/state/settings';
import { cn } from '@/lib/cn';
import { invoke } from '@/lib/ipc';
import { previewProfile, SOUND_PROFILES, type SoundProfileId } from '@/design/sounds';

const ACCENTS = ['teal', 'amber', 'violet', 'rose', 'cyan', 'lime'] as const;
const THEMES = ['obsidian', 'graphite', 'paper'] as const;

/**
 * General settings.
 *
 * Per DESIGN_SYSTEM.md §6.3 chip-style pickers use the .ub-chip class.
 * Active chips use .ub-chip--active (cyan fill, dark text) — never a
 * bordered "selected" ring. This keeps the active-state vocabulary
 * consistent with list selection surfaces (rail) and navigation (rail).
 */
export function GeneralSettings() {
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  if (!s) return null;

  const update = (key: string, value: unknown) =>
    patch({ general: { ...s.general, [key]: value } });

  const g = s.general as Record<string, unknown>;

  return (
    <div className="mx-auto max-w-3xl p-8">
      <PageHeader title="General" />
      <div className="mt-6 space-y-4">
        <Card title="Startup">
          <Row
            label="Launch on login"
            description="Start Utterbar automatically when you sign in to Windows, so the floating bar is always there without opening it manually."
          >
            <Toggle
              checked={Boolean(g.launch_on_login)}
              onChange={(v) => void update('launch_on_login', v)}
            />
          </Row>
          <Row
            label="Start minimised"
            description="When on, Utterbar launches straight to the tray and the floating bar — no main window until you ask for it. When off, the Dashboard opens on launch so you see your stats immediately."
          >
            <Toggle
              checked={Boolean(g.start_minimised)}
              onChange={(v) => void update('start_minimised', v)}
            />
          </Row>
          <Row
            label="Close to tray"
            description="Pressing the X on any window hides it to the tray instead of quitting."
          >
            <Toggle
              checked={Boolean(g.close_to_tray)}
              onChange={(v) => void update('close_to_tray', v)}
            />
          </Row>
        </Card>

        <Card title="Appearance">
          <Row label="Theme" description="Background palette for the main window and onboarding.">
            <div className="flex gap-2">
              {THEMES.map((t) => (
                <button
                  key={t}
                  onClick={() => void update('theme', t)}
                  className={cn(
                    'ub-chip capitalize',
                    s.general.theme === t && 'ub-chip--active',
                  )}
                >
                  {t}
                </button>
              ))}
            </div>
          </Row>
          <Row
            label="Accent"
            description="The colour used for the recording button, the pill glyph, highlights, and the floating bar's waveform."
          >
            <div className="flex gap-2">
              {ACCENTS.map((a) => (
                <button
                  key={a}
                  onClick={() => void update('accent', a)}
                  className={cn(
                    // Raw swatch picker — NOT a ub-chip because the swatch
                    // IS the label. Ring on the active one so the currently
                    // chosen colour is obvious without colouring the border.
                    `h-6 w-6 rounded-full accent-${a} bg-accent transition-shadow duration-180`,
                    s.general.accent === a
                      ? 'ring-2 ring-fg-hi ring-offset-2 ring-offset-bg-1'
                      : 'opacity-70 hover:opacity-100',
                  )}
                  title={a}
                  aria-label={a}
                />
              ))}
            </div>
          </Row>
        </Card>

        <Card title="Feedback">
          <Row
            label="Sound cues"
            description="Play a short synthesised tone when recording starts, stops, succeeds, or errors. The tones are generated in-app via WebAudio so there are no sample files — low-volume by default."
          >
            <Toggle
              checked={Boolean(g.sound_cues_enabled)}
              onChange={(v) => void update('sound_cues_enabled', v)}
            />
          </Row>
          {Boolean(g.sound_cues_enabled) && (
            <Row
              label="Sound profile"
              description="Which family of cue tones to use. Click a profile to hear its success cue."
            >
              <div className="flex flex-wrap items-center gap-2">
                {SOUND_PROFILES.map((p) => {
                  const current =
                    ((g.sound_profile as SoundProfileId | undefined) ?? 'default') === p.id;
                  return (
                    <button
                      key={p.id}
                      onClick={() => {
                        void update('sound_profile', p.id);
                        previewProfile(p.id);
                      }}
                      title={p.description}
                      className={cn('ub-chip', current && 'ub-chip--active')}
                    >
                      {p.label}
                    </button>
                  );
                })}
              </div>
            </Row>
          )}
          <Row
            label="System notifications"
            description="Raise a Windows toast when something actually needs your attention — dictation errors, model-install failures, successful model installs."
          >
            <Toggle
              checked={Boolean(g.notifications_enabled)}
              onChange={(v) => void update('notifications_enabled', v)}
            />
          </Row>
        </Card>

        <Card title="App updates">
          <Row
            label="Check for updates automatically"
            description="Checks the update feed on launch and shows a toast if a newer version is available. No auto-install — you always click Install."
          >
            <Toggle
              checked={Boolean(g.check_updates_automatically)}
              onChange={(v) => void update('check_updates_automatically', v)}
            />
          </Row>
        </Card>

        <ReleaseEmailCard
          current={(g.releases_email as string | null | undefined) ?? null}
          onSubscribe={(email) => void update('releases_email', email)}
        />
      </div>
    </div>
  );
}

/**
 * Release-email subscription card.
 *
 * Opt-in list for the short release-announcement mail we send when a new
 * version ships. The server (utterbar.com) owns the double-opt-in flow
 * (confirmation mail + one-click unsubscribe), so this surface deliberately
 * does NOT track "confirmed vs pending" — we just show the verbatim
 * `message` the server returns and record the email locally so the UI
 * knows the user has opted in across restarts. Unsubscribing happens via
 * the link in every release mail, not here.
 *
 * States:
 *   • Closed, no email on file — toggle off, no inputs.
 *   • Open, fresh — toggle on, email input + Subscribe button.
 *   • Submitted — server's verbatim message under the input.
 *   • Subscribed — read-only "Subscribed as X" line with a change button.
 */
function ReleaseEmailCard({
  current,
  onSubscribe,
}: {
  current: string | null;
  onSubscribe: (email: string | null) => void;
}) {
  const [enabled, setEnabled] = useState<boolean>(Boolean(current));
  const [email, setEmail] = useState<string>(current ?? '');
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<{ tone: 'ok' | 'err'; text: string } | null>(null);

  // Keep local state synced when the underlying setting changes (e.g. reset,
  // import of settings, cross-window update). Without this the card stays
  // on stale values after the store rehydrates.
  useEffect(() => {
    setEnabled(Boolean(current));
    setEmail(current ?? '');
  }, [current]);

  async function submit() {
    const trimmed = email.trim();
    if (!trimmed) {
      setStatus({ tone: 'err', text: 'Enter your email to subscribe.' });
      return;
    }
    setBusy(true);
    setStatus(null);
    try {
      const msg = await invoke<string>('subscribe_to_release_updates', { email: trimmed });
      // Persist locally so the card renders "subscribed" on next load — the
      // confirmation handshake is a server concern so we treat a successful
      // POST as "the user has opted in as far as this app knows".
      onSubscribe(trimmed);
      setStatus({ tone: 'ok', text: msg });
    } catch (e) {
      const env = e as { kind?: string; code?: string; message?: string };
      setStatus({
        tone: 'err',
        text: env?.message ?? 'Could not reach the server. Try again in a moment.',
      });
    } finally {
      setBusy(false);
    }
  }

  function toggle(next: boolean) {
    setEnabled(next);
    if (!next) {
      // Local opt-out only — the server list is authoritative and the user
      // can fully unsubscribe via the link in any release mail. Clearing
      // the local hint just hides the subscribed state in this UI.
      onSubscribe(null);
      setStatus(null);
    }
  }

  const subscribed = Boolean(current) && current === email.trim();

  return (
    <Card
      title="Release updates"
      description="Get a short email when a new Utterbar version ships — what's new, how to update, nothing else. Unsubscribe in one click from any release mail."
    >
      <Row
        label="Email me when there's a new release"
        description="We send at most one mail per release. Your address stays on the server's list only; we never share it."
      >
        <Toggle checked={enabled} onChange={toggle} />
      </Row>
      {enabled && (
        <div className="border-t border-border-soft/70 pt-4">
          {subscribed ? (
            <div className="flex items-center justify-between gap-3">
              <div className="flex min-w-0 items-center gap-2 text-[13px] text-fg-hi">
                <Mail className="h-4 w-4 text-accent shrink-0" />
                <span className="truncate">Subscribed as {current}</span>
              </div>
              <button
                type="button"
                onClick={() => setEmail('')}
                className="ub-btn ub-btn-plain !py-[0.4rem] !text-[12px] shrink-0"
              >
                Change email
              </button>
            </div>
          ) : (
            <div className="flex items-center gap-2">
              <Mail className="h-4 w-4 text-muted shrink-0" />
              <Input
                type="email"
                placeholder="you@example.com"
                value={email}
                disabled={busy}
                onChange={(e) => setEmail(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !busy) void submit();
                }}
                className="flex-1"
                autoComplete="email"
              />
              <button
                type="button"
                onClick={() => void submit()}
                disabled={busy || email.trim().length === 0}
                className="ub-btn ub-btn-cta !py-[0.4rem] !text-[12px] shrink-0"
              >
                {busy ? 'Sending…' : 'Subscribe'}
              </button>
            </div>
          )}
          {status && (
            <p
              className={cn(
                'mt-2 text-[12px] leading-relaxed',
                status.tone === 'ok' ? 'text-accent' : 'text-rose-400',
              )}
            >
              {status.text}
            </p>
          )}
        </div>
      )}
    </Card>
  );
}
