import { useState } from 'react';

import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { invoke } from '@/lib/ipc';
import { useSettings } from '@/state/settings';
import type { HotkeyCombo } from '@/generated/HotkeyCombo';
import type { MicMutePerApp } from '@/generated/MicMutePerApp';

/**
 * Experimental Teams + Discord mic-mute card. Renders inside Settings,
 * Advanced under the existing "Experimental" / Diagnostics block. Two
 * per-app rows (Teams, Discord), each with its own toggle and a
 * click-to-record hotkey input.
 *
 * Per the v1 spec at
 * docs/decisions/true-per-app-microphone-mute-v1-spec.md:
 *
 * - Toggle is off by default. Enabling it does nothing useful until the
 *   user has captured the call app's mute hotkey via the input below.
 * - Discord ships with NO default global mute hotkey (this is a
 *   departure from the original spec, surfaced by Branch 2 testing
 *   against Jay's real Discord). The helper line for Discord points
 *   the user at User Settings, Keybinds, Toggle Mute and explicitly
 *   says they must set one there before Utterbar can do anything.
 * - Validation rejects bare letter / digit / punctuation keys (would
 *   trigger inside dictations), but ALLOWS bare F-keys (F13-F24) and
 *   other non-printable keys, because power users routinely bind the
 *   call-app mute to a single F-key.
 */
export function MicMuteCard() {
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);

  if (!s) return null;

  const exp = s.experimental;
  const teams = exp.mic_mute.teams;
  const discord = exp.mic_mute.discord;

  // Pair-equality check is local to the component, no round-trip needed.
  const pairConflict =
    teams.hotkey != null &&
    discord.hotkey != null &&
    hotkeysEqual(teams.hotkey, discord.hotkey);

  const updatePerApp = (app: 'teams' | 'discord', next: MicMutePerApp) => {
    patch({
      experimental: {
        ...exp,
        mic_mute: {
          ...exp.mic_mute,
          [app]: next,
        },
      },
    });
  };

  return (
    <Card
      eyebrow="EXPERIMENTAL"
      title="Mute your microphone in conferencing apps while recording"
      description={
        'Experimental. This may fail silently. When you start a dictation, ' +
        'Utterbar sends each enabled app’s mute hotkey to its window so the ' +
        'microphone there is muted, then sends it again when you stop to unmute. ' +
        'Each app needs you to confirm what its mute hotkey is, because both Teams ' +
        'and Discord let you rebind it. The defaults below match each app’s ' +
        'out-of-the-box shortcut. After every dictation, a banner asks you to ' +
        'confirm the mute state in the relevant app, because Utterbar cannot read ' +
        'either app’s actual mute state and shouldn’t pretend it can. If ' +
        'this ever misbehaves, turn off the relevant toggle and use the call ' +
        'app’s own mute button.'
      }
    >
      <MicMuteAppRow
        appKey="teams"
        appLabel="Microsoft Teams"
        helperText={
          'If you’ve changed the mute hotkey in Teams, click "Re-record" and ' +
          'press your current Toggle Mute shortcut. Default in Teams: Ctrl+Shift+M. ' +
          'You can verify the current Teams binding under Teams Settings, Keyboard ' +
          'shortcuts, Toggle mute.'
        }
        per={teams}
        otherHotkey={discord.hotkey}
        onChange={(next) => updatePerApp('teams', next)}
      />
      <MicMuteAppRow
        appKey="discord"
        appLabel="Discord"
        helperText={
          'Discord does not ship with a default global mute hotkey. Open Discord, ' +
          'go to User Settings, Keybinds, click Add a Keybind, choose "Toggle Mute", ' +
          'and pick a key. Then click "Re-record" below and press the same key here ' +
          'so Utterbar sends what Discord is listening for.'
        }
        per={discord}
        otherHotkey={teams.hotkey}
        onChange={(next) => updatePerApp('discord', next)}
      />

      {pairConflict && (
        <div className="mt-3 rounded-input border border-danger/40 bg-danger/10 px-3 py-2 text-[12px] text-danger">
          Teams and Discord cannot share the same hotkey. Pick a different one
          for one of them.
        </div>
      )}

      <div className="mt-5 ub-eyebrow">Supported apps</div>
      <table className="mt-2 w-full text-[12px]">
        <thead>
          <tr className="text-left text-muted">
            <th className="py-1 font-medium">App</th>
            <th className="py-1 font-medium">Status</th>
            <th className="py-1 font-medium">Tested against</th>
          </tr>
        </thead>
        <tbody className="text-fg">
          <tr className="border-t border-border-soft/70">
            <td className="py-1.5">Microsoft Teams (classic and new Teams)</td>
            <td className="py-1.5 text-fg-hi">Experimental</td>
            <td className="py-1.5 text-muted">
              Last verified 2026-05-04 against Teams classic 1.7.x and new
              Teams 24.x
            </td>
          </tr>
          <tr className="border-t border-border-soft/70">
            <td className="py-1.5">Discord (stable, PTB, Canary)</td>
            <td className="py-1.5 text-fg-hi">Experimental</td>
            <td className="py-1.5 text-muted">
              Last verified 2026-05-04 against Discord stable 1.0.90xx
            </td>
          </tr>
        </tbody>
      </table>
    </Card>
  );
}

function MicMuteAppRow(props: {
  appKey: 'teams' | 'discord';
  appLabel: string;
  helperText: string;
  per: MicMutePerApp;
  otherHotkey: HotkeyCombo | null;
  onChange: (next: MicMutePerApp) => void;
}) {
  return (
    <Row
      label={props.appLabel}
      description={
        <div className="space-y-2">
          <div>
            <span className="text-muted text-[12px]">Mute hotkey:&nbsp;</span>
            <HotkeyCaptureInput
              value={props.per.hotkey}
              otherHotkey={props.otherHotkey}
              onChange={(next) =>
                props.onChange({ ...props.per, hotkey: next })
              }
            />
          </div>
          <div className="text-[11px] leading-relaxed text-muted">
            {props.helperText}
          </div>
        </div>
      }
      align="start"
    >
      <Toggle
        checked={props.per.enabled}
        onChange={(v) => props.onChange({ ...props.per, enabled: v })}
      />
    </Row>
  );
}

/**
 * Click-to-record hotkey capture. While not recording, displays the
 * captured combo (or "(not set)" if none) plus a Re-record button.
 * While recording, shows "Press your hotkey..." and listens for the
 * next keydown event with at least one main key. The captured combo
 * is validated via the Tauri command before being committed; an
 * inline error appears if the combo is rejected.
 */
function HotkeyCaptureInput({
  value,
  otherHotkey,
  onChange,
}: {
  value: HotkeyCombo | null;
  otherHotkey: HotkeyCombo | null;
  onChange: (next: HotkeyCombo | null) => void;
}) {
  const [recording, setRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const startRecording = () => {
    setError(null);
    setRecording(true);
  };

  const handleKeyDown = async (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();

    if (e.key === 'Escape') {
      setRecording(false);
      setError(null);
      return;
    }

    // Pure-modifier keypress (e.g. just Shift) is ignored; we wait for
    // the user to also press a main key.
    if (
      e.key === 'Control' ||
      e.key === 'Shift' ||
      e.key === 'Alt' ||
      e.key === 'Meta'
    ) {
      return;
    }

    const vk = browserKeyToVk(e.key, e.code);
    if (vk === 0) {
      setError('Key not recognised. Try a letter, digit, or function key.');
      return;
    }

    const combo: HotkeyCombo = {
      ctrl: e.ctrlKey,
      shift: e.shiftKey,
      alt: e.altKey,
      win: e.metaKey,
      vk,
      label: formatHotkeyLabel({
        ctrl: e.ctrlKey,
        shift: e.shiftKey,
        alt: e.altKey,
        win: e.metaKey,
        vk,
        label: '', // recomputed by formatter
      }),
    };

    // Pair-conflict check: client-side, no round-trip.
    if (otherHotkey && hotkeysEqual(combo, otherHotkey)) {
      setError(
        'Teams and Discord cannot share the same hotkey. Pick a different one.',
      );
      return;
    }

    // Single-combo validation via the Rust command.
    try {
      await invoke<void>('validate_mic_mute_hotkey', {
        combo,
        utterbarShortcutConflicts: [], // TODO Branch 3c: pass current Utterbar global hotkeys
      });
    } catch (reason) {
      const msg = (reason as string) || 'invalid';
      setError(validationErrorMessage(msg));
      return;
    }

    setRecording(false);
    setError(null);
    onChange(combo);
  };

  return (
    <span className="inline-flex items-center gap-2">
      <div
        role="button"
        tabIndex={0}
        onKeyDown={handleKeyDown}
        onClick={() => {
          if (!recording) startRecording();
        }}
        className={`inline-flex min-w-[140px] items-center justify-center rounded-input px-3 py-1 text-[12px] font-mono transition-colors ${
          recording
            ? 'bg-accent/20 text-accent ring-1 ring-accent'
            : value
              ? 'bg-bg-2 text-fg-hi hover:bg-bg-2/70 cursor-pointer'
              : 'bg-bg-2/50 text-muted italic hover:bg-bg-2/70 cursor-pointer'
        }`}
      >
        {recording ? 'Press your hotkey...' : value ? value.label : '(not set)'}
      </div>
      {!recording && (
        <button
          type="button"
          onClick={startRecording}
          className="text-[11px] text-muted hover:text-fg-hi"
        >
          {value ? 'Re-record' : 'Record'}
        </button>
      )}
      {!recording && value && (
        <button
          type="button"
          onClick={() => {
            setError(null);
            onChange(null);
          }}
          className="text-[11px] text-muted hover:text-danger"
        >
          Clear
        </button>
      )}
      {error && (
        <span className="text-[11px] text-danger ml-2">{error}</span>
      )}
    </span>
  );
}

// =============================================================================
// Helpers
// =============================================================================

function hotkeysEqual(a: HotkeyCombo, b: HotkeyCombo): boolean {
  return (
    a.vk === b.vk &&
    a.ctrl === b.ctrl &&
    a.shift === b.shift &&
    a.alt === b.alt &&
    a.win === b.win
  );
}

function formatHotkeyLabel(combo: HotkeyCombo): string {
  const parts: string[] = [];
  if (combo.ctrl) parts.push('Ctrl');
  if (combo.shift) parts.push('Shift');
  if (combo.alt) parts.push('Alt');
  if (combo.win) parts.push('Win');
  parts.push(vkToDisplayName(combo.vk));
  return parts.join('+');
}

/**
 * Map a browser KeyboardEvent.key + KeyboardEvent.code to a Win32
 * virtual-key code (`u16`). Covers the common cases: A-Z, 0-9, F1-F24,
 * arrow keys, Pause / PrtSc / ScrollLock. Returns 0 if the key isn't
 * mappable (caller surfaces an error).
 */
function browserKeyToVk(key: string, code: string): number {
  // Letters A-Z: VK code is the ASCII upper-case value.
  if (/^[a-zA-Z]$/.test(key)) {
    return key.toUpperCase().charCodeAt(0);
  }
  // Top-row digits 0-9: VK code is ASCII '0' through '9' (0x30-0x39).
  if (/^[0-9]$/.test(key)) {
    return key.charCodeAt(0);
  }
  // F-keys F1-F24: VK_F1 = 0x70, VK_F2 = 0x71, ..., VK_F24 = 0x87.
  const fMatch = /^F(\d{1,2})$/.exec(key);
  if (fMatch) {
    const n = Number(fMatch[1]);
    if (n >= 1 && n <= 24) {
      return 0x70 + (n - 1);
    }
  }
  // Named non-printable keys we explicitly accept as bare-key bindings.
  switch (key) {
    case 'Pause':
      return 0x13;
    case 'PrintScreen':
      return 0x2c;
    case 'ScrollLock':
      return 0x91;
    case 'Insert':
      return 0x2d;
    case 'Delete':
      return 0x2e;
    case 'Home':
      return 0x24;
    case 'End':
      return 0x23;
    case 'PageUp':
      return 0x21;
    case 'PageDown':
      return 0x22;
    default:
      break;
  }
  // Numpad keys via KeyboardEvent.code.
  const numpadMatch = /^Numpad(\d)$/.exec(code);
  if (numpadMatch) {
    return 0x60 + Number(numpadMatch[1]);
  }
  return 0;
}

/**
 * Reverse mapping for display. Covers the same set as `browserKeyToVk`.
 * Falls back to a `VK ${vk}` string for codes outside the known set;
 * the user shouldn't see this but if they do it's diagnosable.
 */
function vkToDisplayName(vk: number): string {
  if (vk >= 0x41 && vk <= 0x5a) return String.fromCharCode(vk);
  if (vk >= 0x30 && vk <= 0x39) return String.fromCharCode(vk);
  if (vk >= 0x70 && vk <= 0x87) return `F${vk - 0x70 + 1}`;
  if (vk >= 0x60 && vk <= 0x69) return `Numpad${vk - 0x60}`;
  switch (vk) {
    case 0x13:
      return 'Pause';
    case 0x2c:
      return 'PrtSc';
    case 0x91:
      return 'ScrollLock';
    case 0x2d:
      return 'Insert';
    case 0x2e:
      return 'Delete';
    case 0x24:
      return 'Home';
    case 0x23:
      return 'End';
    case 0x21:
      return 'PageUp';
    case 0x22:
      return 'PageDown';
    default:
      return `VK ${vk}`;
  }
}

function validationErrorMessage(reason: string): string {
  switch (reason) {
    case 'bare_character_key':
      return 'Pick a hotkey with at least one modifier (Ctrl, Shift, Alt, or Win) so you don’t trigger it by accident while typing. Function keys (F13-F24) are also fine.';
    case 'no_main_key':
      return 'Modifier keys alone are not a valid hotkey. Press a main key too.';
    case 'conflicts_with_utterbar_shortcut':
      return 'That combo is already bound to one of Utterbar’s own shortcuts. Pick a different one.';
    default:
      return 'That hotkey is not allowed. Pick a different one.';
  }
}
