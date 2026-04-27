import { useEffect, useMemo, useRef, useState } from 'react';
import { AlertTriangle, Check, Copy, Eye, EyeOff, ExternalLink, Laptop, ShieldX } from 'lucide-react';
import { open as openShell } from '@tauri-apps/plugin-shell';

import { Button } from '@/components/common/Button';
import { Card } from '@/components/common/Card';
import { Input } from '@/components/common/Input';
import { Led } from '@/components/common/Led';
import { PageHeader } from '@/components/common/PageHeader';
import { invoke } from '@/lib/ipc';
import { cn } from '@/lib/cn';
import type { LicenceDevice } from '@/generated/LicenceDevice';
import type { LicenceStatus } from '@/generated/LicenceStatus';
import {
  toLicenceError,
  useLicence,
  type LicenceError,
  type LicenceErrorCode,
} from '@/state/licence';

const PRICING_URL = 'https://utterbar.com/pricing';
const ACCOUNT_URL = 'https://utterbar.com/account';

/**
 * Auto-rehide window when the licence key is revealed via the eye toggle.
 * 30 s is enough to read it aloud or copy it manually, short enough that
 * an unattended screen doesn't sit unmasked indefinitely.
 */
const REVEAL_TIMEOUT_MS = 30_000;

/**
 * Mask a licence key for display, preserving the dash structure so the
 * field doesn't visibly resize when toggled.
 *
 * Real key shape: `UB-XXXX-XXXX-XXXX-XXXX-XXXX` (the `UB-` prefix is
 * brand-recognisable and worth keeping visible; only the random parts
 * carry sensitive entropy).
 *
 * Implementation: replace every char after the first dash with `•`. The
 * first dash itself and all subsequent dashes pass through, so the
 * groups stay visually aligned with the unmasked form.
 */
function maskLicenceKey(key: string): string {
  // Keep the first segment (typically "UB") then mask everything after.
  const firstDash = key.indexOf('-');
  if (firstDash < 0) {
    // No dash in the key — fall back to dotting everything.
    return '•'.repeat(key.length);
  }
  const head = key.slice(0, firstDash + 1); // includes the dash itself
  const tail = key.slice(firstDash + 1);
  return head + tail.replace(/[^-]/g, '•');
}
// 2026-04-27: outbound email moved off utterbar.com to the studio
// domain ddraig.dev. The website is still utterbar.com (PRICING_URL,
// ACCOUNT_URL above) — only the support inbox changed.
const SUPPORT_EMAIL = 'hello@ddraig.dev';

/**
 * Settings → Licence.
 *
 * Per the product brief this is NOT an "Account" screen, there's no account
 * in Utterbar, only a licence that unlocks Pro. Two rendered states:
 *
 *   Free , paste UB-XXXX-… → activate. Inline errors per server slug.
 *   Pro  , shows current key (copyable), this-device metadata, and a list
 *           of other devices pulled fresh on the last /activate. Managing
 *           those other devices is deliberately pushed to utterbar.com/account
 *           because the Rust `licence_deactivate` command only targets the
 *           current device.
 *
 * Design hooks (DESIGN_SYSTEM.md):
 *   §4.5 , the Pro card uses `rail` to signal "featured / active state".
 *   §6.1 , paste input sunken on bg-0 via <Input>.
 *   §8.1 , tier pill reuses the LED primitive; cyan for Pro, muted for Free.
 *   §11.4, active list items lift to bg-2; "this device" row follows that.
 */
export function LicenceSettings() {
  const status = useLicence((s) => s.status);
  const loaded = useLicence((s) => s.loaded);

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader
        title="Licence"
        subtitle="Your Utterbar Pro key unlocks advanced features on this device. No account, no sign-in, just the key."
        actions={<TierBadge status={status} />}
      />

      {/* `loaded` gates the render so we don't flash "Free" for a frame on
          mount before the store hydrates. */}
      {!loaded ? (
        <div className="mt-8 text-[13px] text-muted">Loading…</div>
      ) : status?.tier === 'pro' ? (
        <ProPanel status={status} />
      ) : (
        <FreePanel />
      )}
    </div>
  );
}

/* ------------------------------------------------------------------ Tier pill */

/**
 * Compact status indicator used in the page header. Pro = glowing cyan LED,
 * Free = quiet pill. Built around the `Led` primitive so the glow recipe
 * stays consistent with the recording-state LED elsewhere in the app.
 */
function TierBadge({ status }: { status: LicenceStatus | null }) {
  if (status?.tier === 'pro') {
    return (
      <span className="inline-flex items-center gap-2 rounded-full bg-bg-2 px-3 py-1.5 text-[12px] font-medium text-fg-hi">
        <Led size={6} />
        Pro · {friendlySku(status.sku)}
      </span>
    );
  }
  return (
    <span className="inline-flex items-center gap-2 rounded-full bg-bg-2 px-3 py-1.5 text-[12px] font-medium text-muted">
      Free
    </span>
  );
}

/* ------------------------------------------------------------------ Free state */

function FreePanel() {
  const [key, setKey] = useState('');
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<LicenceError | null>(null);
  const trimmed = key.trim();

  async function activate() {
    if (!trimmed || busy) return;
    setBusy(true);
    setErr(null);
    try {
      // The store updates via the `licence_changed` event, we don't need
      // to set it from the return value, but holding the call in busy state
      // keeps the button from double-firing mid-flight.
      await invoke('licence_activate', { key: trimmed });
      setKey('');
    } catch (e) {
      setErr(toLicenceError(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <WhatProUnlocks />
      <Card className="mt-4">
      <div className="ub-eyebrow mb-3">Activate Pro</div>
      <p className="text-[14px] leading-[1.6] text-muted max-w-[56ch]">
        Paste the licence key from your purchase email. Activation takes a second and uses
        the internet once, after that, Utterbar verifies locally on every launch.
      </p>

      <form
        className="mt-5 flex flex-col gap-3 sm:flex-row sm:items-start"
        onSubmit={(e) => {
          e.preventDefault();
          void activate();
        }}
      >
        <Input
          value={key}
          onChange={(e) => setKey(e.target.value)}
          placeholder="UB-XXXX-XXXX-XXXX-XXXX-XXXX"
          // Geist Mono on the input, this is literal data, exactly the case
          // the spec reserves mono for (§2.4).
          className="font-mono text-[14px] tracking-[0.02em] sm:flex-1"
          autoComplete="off"
          autoCorrect="off"
          spellCheck={false}
          disabled={busy}
          aria-invalid={err ? true : undefined}
        />
        <Button
          type="submit"
          variant="cta"
          disabled={!trimmed || busy}
          className="sm:shrink-0"
        >
          {busy ? 'Activating…' : 'Activate'}
        </Button>
      </form>

      {err && <ErrorSurface err={err} onDismiss={() => setErr(null)} />}

      <div className="mt-6 border-t border-border-soft/70 pt-4 text-[13px] text-muted">
        Don't have a key?{' '}
        <button
          onClick={() => void openShell(PRICING_URL).catch(() => undefined)}
          className="inline-flex items-center gap-1 text-fg-hi underline decoration-border-strong decoration-[1.5px] underline-offset-[3px] transition-colors hover:text-accent hover:decoration-accent"
        >
          Buy Utterbar Pro
          <ExternalLink className="h-3 w-3" />
        </button>
      </div>
    </Card>

    <TipCard />
    </>
  );
}

/**
 * Tip jar surface, shown only on the Free state of the Licence page.
 *
 * The Pro state intentionally does not show this, Pro users have already
 * paid £39.99 and surfacing a "tip the developer" prompt to them would
 * read as cap-in-hand. Free users have a clean, no-strings way to
 * support the project if they want to. Buttons open the website's
 * `/tip/start?amount=...` URL in the user's default browser via Tauri's
 * shell plugin; payment lives entirely on Stripe-hosted Checkout.
 *
 * Auto-hides if the website hasn't got `STRIPE_TIP_PRICE_ID` set, but
 * the failure mode of "user clicks, lands on a blank page" doesn't
 * surface here because the website redirects home in that case.
 */
function TipCard() {
  const startTip = (amountPence: number | null) => {
    const url = amountPence
      ? `https://utterbar.com/tip/start?amount=${amountPence}`
      : `https://utterbar.com/tip/start`;
    void openShell(url).catch(() => undefined);
  };

  return (
    <Card className="mt-4">
      <div className="ub-eyebrow mb-3">Tip the developer</div>
      <p className="text-[13px] leading-[1.55] text-muted max-w-[56ch]">
        Utterbar is free. If it's earning its keep on your machine, a few quid keeps the
        next release moving. Voluntary, no perks, no licence, just thank-you money. Opens
        Stripe in your browser.
      </p>
      <div className="mt-4 flex flex-wrap items-center gap-2">
        {[300, 500, 1000].map((pence) => (
          <button
            key={pence}
            type="button"
            onClick={() => startTip(pence)}
            className="rounded-input bg-bg-2 px-3 py-1.5 text-[12px] font-medium text-fg-hi transition-colors hover:bg-bg-2/70"
          >
            £{pence / 100}
          </button>
        ))}
        <button
          type="button"
          onClick={() => startTip(null)}
          className="rounded-input bg-bg-2 px-3 py-1.5 text-[12px] font-medium text-fg-hi transition-colors hover:bg-bg-2/70"
        >
          Custom →
        </button>
      </div>
    </Card>
  );
}

/**
 * Free → Pro feature explainer.
 *
 * Sits above the activation card on the Free state. Concrete and compact:
 * the five things Pro actually unlocks today, with the Free cap quoted
 * next to each one so the value is unambiguous. Bullets, not marketing
 * copy.
 *
 * Caps quoted here MUST track `src-tauri/src/util/gating.rs`. If we
 * change a cap there, change it here. There's no shared TS source for
 * these numbers, they're display strings, the backend is the
 * enforcement.
 */
function WhatProUnlocks() {
  const items = [
    {
      title: 'Unlimited custom modes',
      free: '3 modes on Free',
    },
    {
      title: 'Unlimited vocabulary & replacements',
      free: '50 words / 25 replacements on Free',
    },
    {
      title: 'File transcription',
      free: 'Pro only, drop a .wav / .mp3 / .m4a in and get a transcript',
    },
    {
      title: 'Full history with re-transcribe',
      free: 'Free keeps the last 7 days; Pro keeps everything',
    },
    {
      title: 'High-accuracy GPU model (fp16)',
      free: 'Free uses the compact model; Pro adds the larger fp16 build for stronger accuracy on capable GPUs',
    },
  ];
  return (
    <Card className="mt-6" rail>
      <div className="ub-eyebrow mb-2">What Pro unlocks</div>
      <p className="text-[13px] leading-[1.55] text-muted max-w-[58ch]">
        One-time £39.99 GBP. No subscription. Three-device cap, transferable any time
        from utterbar.com/account.
      </p>
      <ul className="mt-4 flex flex-col gap-2.5">
        {items.map((it) => (
          <li key={it.title} className="flex items-start gap-2.5">
            <Check className="mt-[3px] h-3.5 w-3.5 shrink-0 text-accent" />
            <div className="flex-1">
              <div className="text-[13px] font-medium text-fg-hi">{it.title}</div>
              <div className="text-[12px] leading-snug text-muted">{it.free}</div>
            </div>
          </li>
        ))}
      </ul>
    </Card>
  );
}

/* ------------------------------------------------------------------ Pro state */

function ProPanel({
  status,
}: {
  status: Extract<LicenceStatus, { tier: 'pro' }>;
}) {
  const [copied, setCopied] = useState(false);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<LicenceError | null>(null);

  // Licence key is masked by default. Eye toggle reveals; auto-rehides
  // after REVEAL_TIMEOUT_MS so an unattended screen doesn't leak the key.
  // The timer ref lives across renders so we can cancel it when the user
  // manually re-hides (avoids a stale rehide firing after they've already
  // toggled state again).
  const [revealed, setRevealed] = useState(false);
  const revealTimeoutRef = useRef<number | null>(null);

  function clearRevealTimer() {
    if (revealTimeoutRef.current !== null) {
      window.clearTimeout(revealTimeoutRef.current);
      revealTimeoutRef.current = null;
    }
  }

  function toggleReveal() {
    clearRevealTimer();
    if (revealed) {
      setRevealed(false);
    } else {
      setRevealed(true);
      revealTimeoutRef.current = window.setTimeout(() => {
        setRevealed(false);
        revealTimeoutRef.current = null;
      }, REVEAL_TIMEOUT_MS);
    }
  }

  // Cancel any pending rehide on unmount, otherwise the timer fires
  // against a stale setRevealed and React warns about state on an
  // unmounted component.
  useEffect(() => clearRevealTimer, []);

  const { this_device, other_devices } = useMemo(() => partitionDevices(status.devices), [
    status.devices,
  ]);

  async function copyKey() {
    try {
      await navigator.clipboard.writeText(status.display_key);
      setCopied(true);
      setTimeout(() => setCopied(false), 1400);
    } catch {
      // Silent, the display key is visible on screen; copying is a
      // convenience, not a dependency.
    }
  }

  async function deactivateHere() {
    if (busy) return;
    const ok = window.confirm(
      'Remove Pro from this device? You can re-activate using the same key from Settings → Licence.',
    );
    if (!ok) return;
    setBusy(true);
    setErr(null);
    try {
      await invoke('licence_deactivate');
    } catch (e) {
      setErr(toLicenceError(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="mt-6 flex flex-col gap-5">
      {/* The primary Pro card carries the cyan rail, this is the "active /
          featured" state and the spec earmarks the rail for exactly that. */}
      <Card rail>
        <div className="ub-eyebrow mb-2">Your licence</div>
        <div className="flex items-center gap-3">
          <code
            className="font-mono text-[15px] tracking-[0.02em] text-fg-hi break-all"
            // aria-label always uses the real key so screen readers can
            // hear it on focus regardless of visual masking; sighted
            // users see the masked form. The masked form is decorative.
            aria-label={`Licence key ${status.display_key}`}
          >
            {revealed ? status.display_key : maskLicenceKey(status.display_key)}
          </code>
          <button
            onClick={toggleReveal}
            className="group inline-flex items-center gap-1.5 rounded-[8px] px-2 py-1 text-[12px] font-medium text-muted transition-colors hover:bg-bg-2 hover:text-fg-hi"
            aria-label={revealed ? 'Hide licence key' : 'Show licence key'}
            aria-pressed={revealed}
            title={revealed ? 'Hide' : 'Show'}
          >
            {revealed ? (
              <>
                <EyeOff className="h-3.5 w-3.5" />
                Hide
              </>
            ) : (
              <>
                <Eye className="h-3.5 w-3.5" />
                Show
              </>
            )}
          </button>
          <button
            onClick={() => void copyKey()}
            className="group inline-flex items-center gap-1.5 rounded-[8px] px-2 py-1 text-[12px] font-medium text-muted transition-colors hover:bg-bg-2 hover:text-fg-hi"
            aria-label="Copy licence key"
          >
            {copied ? (
              <>
                <Check className="h-3.5 w-3.5 text-accent" />
                Copied
              </>
            ) : (
              <>
                <Copy className="h-3.5 w-3.5" />
                Copy
              </>
            )}
          </button>
        </div>
        <div className="mt-2 font-mono text-[11px] uppercase tracking-[0.14em] text-fg-dim">
          ID {status.licence_id} · {status.sku}
        </div>
      </Card>

      {/* This device, lift to bg-2 so it reads as "the active row" per §11.4. */}
      <Card>
        <div className="ub-eyebrow mb-3">This device</div>
        <div className="flex items-center justify-between gap-4 rounded-[12px] bg-bg-2 p-4">
          <div className="flex items-center gap-3 min-w-0">
            <div
              className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-bg-1 text-accent"
              aria-hidden
            >
              <Laptop className="h-4 w-4" />
            </div>
            <div className="min-w-0">
              <div className="truncate text-[14px] font-medium text-fg-hi">
                {this_device?.device_name ?? 'This device'}
              </div>
              <div className="mt-0.5 font-mono text-[11px] text-muted">
                {this_device
                  ? `Activated ${formatDate(this_device.first_seen)}`
                  : 'Activated'}
              </div>
            </div>
          </div>
          <Button variant="danger" size="sm" onClick={() => void deactivateHere()} disabled={busy}>
            {busy ? 'Removing…' : 'Sign out here'}
          </Button>
        </div>

        {err && <ErrorSurface err={err} onDismiss={() => setErr(null)} />}
      </Card>

      {/* Other devices, read-only. The Rust deactivate command only targets
          the fingerprint of *this* install; cross-device deactivation is a
          website concern, so we link out. Max-devices gets shown here even if
          the list is empty so the user can see the cap at a glance. */}
      <Card>
        <div className="mb-3 flex items-center justify-between">
          <div className="ub-eyebrow">Other devices</div>
          <div className="font-mono text-[11px] text-muted">
            {other_devices.length} / {status.max_devices === 0 ? '∞' : status.max_devices - 1}
          </div>
        </div>

        {other_devices.length === 0 ? (
          <p className="text-[13px] text-muted">
            No other devices are currently activated with this licence.
          </p>
        ) : (
          <ul className="flex flex-col gap-2">
            {other_devices.map((d) => (
              <li
                key={d.fingerprint}
                className="flex items-center justify-between gap-4 rounded-[10px] px-3 py-2.5 text-[13px] transition-colors hover:bg-bg-2/60"
              >
                <div className="min-w-0">
                  <div className="truncate font-medium text-fg-hi">{d.device_name}</div>
                  <div className="mt-0.5 font-mono text-[11px] text-muted">
                    Activated {formatDate(d.first_seen)}
                  </div>
                </div>
              </li>
            ))}
          </ul>
        )}

        <div className="mt-4 border-t border-border-soft/70 pt-3">
          <button
            onClick={() => void openShell(ACCOUNT_URL).catch(() => undefined)}
            className="inline-flex items-center gap-1.5 text-[13px] text-muted transition-colors hover:text-accent"
          >
            Manage devices online
            <ExternalLink className="h-3.5 w-3.5" />
          </button>
        </div>
      </Card>
    </div>
  );
}

/* ------------------------------------------------------------------ Error surface */

/**
 * Inline error renderer. Keeps all the code-slug → user-string mapping in one
 * place, colour-coded by severity. `revoked` is the only case that escalates
 * to the rose banner, everything else is an informational inline notice,
 * because the distinction we care about is "key is bad now" vs "try again".
 */
function ErrorSurface({ err, onDismiss }: { err: LicenceError; onDismiss: () => void }) {
  const spec = ERROR_SPEC[err.code] ?? {
    tone: 'warn' as const,
    title: 'Something went wrong.',
    body: err.message,
  };
  const body = spec.body === null ? err.message : spec.body;
  const isRevoked = err.code === 'revoked';

  return (
    <div
      role={isRevoked ? 'alert' : 'status'}
      className={cn(
        'mt-4 flex gap-3 rounded-[10px] px-4 py-3 text-[13px]',
        isRevoked
          ? 'border border-rose/40 bg-rose/10 text-fg-hi'
          : 'border border-border-soft/70 bg-bg-2 text-fg',
      )}
    >
      <div aria-hidden className={cn('mt-0.5 shrink-0', isRevoked ? 'text-rose' : 'text-amber')}>
        {isRevoked ? <ShieldX className="h-4 w-4" /> : <AlertTriangle className="h-4 w-4" />}
      </div>
      <div className="flex-1 min-w-0">
        <div className="font-medium text-fg-hi">{spec.title}</div>
        {body && <div className="mt-0.5 text-muted">{body}</div>}
        {spec.action && (
          <div className="mt-2 flex flex-wrap items-center gap-3">{spec.action}</div>
        )}
      </div>
      <button
        onClick={onDismiss}
        className="text-muted transition-colors hover:text-fg-hi"
        aria-label="Dismiss"
      >
        ×
      </button>
    </div>
  );
}

type ErrorSpec = {
  tone: 'warn' | 'error';
  title: string;
  /** If `null`, fall back to the raw server message. */
  body: string | null;
  action?: React.ReactNode;
};

const ERROR_SPEC: Record<LicenceErrorCode, ErrorSpec> = {
  malformed: {
    tone: 'warn',
    title: "That key doesn't look right.",
    body: 'Check for typos, licence keys are 20 characters, digits plus A–Z (not I, L, O, or U).',
  },
  not_found: {
    tone: 'warn',
    title: "We couldn't find that licence.",
    body: "Double-check the key from your purchase email. If it still doesn't work, get in touch.",
    action: <SupportMail />,
  },
  revoked: {
    tone: 'error',
    title: 'This licence has been revoked.',
    body: null, // the server's message is usually the clearer thing here
    action: <SupportMail />,
  },
  device_limit: {
    tone: 'warn',
    title: "You have reached this licence's device limit.",
    body: 'Free up a slot by removing another device from your account, then try again.',
    action: (
      <button
        onClick={() => void openShell(ACCOUNT_URL).catch(() => undefined)}
        className="inline-flex items-center gap-1.5 text-[12px] font-medium text-accent transition-colors hover:text-accent-2"
      >
        Manage devices
        <ExternalLink className="h-3 w-3" />
      </button>
    ),
  },
  network: {
    tone: 'warn',
    title: "We couldn't reach Utterbar's servers.",
    body: 'Check your internet connection and try again. Activation only needs the network once.',
  },
  verification_failed: {
    tone: 'error',
    title: 'The licence we received failed a security check.',
    body: "This shouldn't happen, please try again, and contact us if it keeps failing.",
    action: <SupportMail />,
  },
  already_activated: {
    tone: 'warn',
    title: 'This device is already activated.',
    body: null,
  },
  rate_limited: {
    tone: 'warn',
    title: "You're trying a lot of keys.",
    body: 'Slow down for a minute and try again, the activation endpoint briefly blocks rapid retries.',
  },
  pro_required: {
    // `pro_required` travels over the same licence-error envelope but is
    // raised by backend gates (file transcription, fp16 install, custom
    // modes, vocab caps, audio retention, re-transcribe) rather than the
    // activation flow. The UI surfaces ProNudge next to the gated control,
    // so the only way we'd ever hit this spec is if a user pasted a pro-
    // gated IPC call into the wrong place, treat it as a non-fatal hint.
    tone: 'warn',
    title: 'That feature needs Pro.',
    body: null,
  },
  internal: {
    tone: 'error',
    title: 'Something went wrong on this device.',
    body: null,
  },
};

function SupportMail() {
  return (
    <a
      href={`mailto:${SUPPORT_EMAIL}`}
      className="inline-flex items-center gap-1.5 text-[12px] font-medium text-accent transition-colors hover:text-accent-2"
    >
      Email support
      <ExternalLink className="h-3 w-3" />
    </a>
  );
}

/* ------------------------------------------------------------------ Helpers */

function partitionDevices(devices: LicenceDevice[]): {
  this_device: LicenceDevice | undefined;
  other_devices: LicenceDevice[];
} {
  const this_device = devices.find((d) => d.this_device);
  const other_devices = devices.filter((d) => !d.this_device);
  return { this_device, other_devices };
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

/** Pretty-print the server SKU slug. Adds friendly names for the SKUs we know;
 *  unknown SKUs fall back to the raw slug so we don't lie about the tier. */
function friendlySku(sku: string): string {
  switch (sku) {
    case 'utterbar-pro-lifetime':
      return 'lifetime';
    case 'utterbar-pro-annual':
      return 'annual';
    default:
      return sku;
  }
}
