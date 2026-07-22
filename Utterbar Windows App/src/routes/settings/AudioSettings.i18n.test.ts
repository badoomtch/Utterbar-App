import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'AudioSettings.tsx'), 'utf8');

// Strip line + block comments before grepping so prose commentary in
// AudioSettings.tsx (which legitimately mentions things like
// "microphone", "noise", or "system audio") doesn't false-trigger the
// guard.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('AudioSettings i18n source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]settings['"]\)/);
  });

  it('does not embed pinned old visible literal props or strings', () => {
    // Narrow allow-list. Each entry is a literal that lived in the
    // pre-localisation source and would re-appear if a future edit
    // bypassed t(). Comments are stripped above so prose commentary
    // mentioning these tokens doesn't false-trigger this guard.
    const forbidden = [
      // PageHeader props.
      'title="Audio"',
      'subtitle="Input devices, preprocessing, and mic testing."',
      // Capture-source card props.
      'title="Capture source"',
      // Per-option chrome that lived inline in the CAPTURE_SOURCES
      // array — pinned at the inline-array level so a regression that
      // re-introduces the array also fails.
      "label: 'Microphone'",
      "label: 'System audio'",
      "label: 'Mixed (mic + system)'",
      // Meeting-Mode callout card props + link text.
      'title="Heads up: Meeting Mode handles this for you"',
      '>Open Modes →<',
      // Input-device card props.
      'title="Input device"',
      'label="Microphone"',
      'label="Live meter"',
      // The Select empty-option label and the per-device caption flag
      // — pre-localisation bare strings.
      "label: 'System default'",
      "? 'system default'",
      // Live-meter button text.
      "? 'Stop' : 'Test'",
      // Noise-handling card props.
      'title="Noise handling"',
      'label="Suppression strength"',
      'label="Between-word gate"',
      'label="Dynamic normalisation"',
      'label="Silence trimming"',
      // The chip-button title attribute (was a backtick-template
      // composing lvl.blurb with " · Click to preview").
      'title={`${lvl.blurb} · Click to preview`}',
      // The preview-hint span text.
      '>Tap any level to hear a ~1 s sample.<',
      // Per-level entries that lived inline in the NOISE_LEVELS array.
      "label: 'Off'",
      "label: 'Light'",
      "label: 'Balanced'",
      "label: 'Aggressive'",
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `AudioSettings.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('routes the page header and visible chrome through audio.* keys', () => {
    expect(SOURCE).toMatch(/audio\.title/);
    expect(SOURCE).toMatch(/audio\.subtitle/);
    expect(SOURCE).toMatch(/audio\.capture_source\.title/);
    expect(SOURCE).toMatch(/audio\.capture_source\.description/);
    // Per-option labels and notes resolved per-id at render time.
    expect(SOURCE).toMatch(/audio\.capture_source\.options\.\$\{[a-zA-Z._]*?\}\.label/);
    expect(SOURCE).toMatch(/audio\.capture_source\.options\.\$\{[a-zA-Z._]*?\}\.note/);
    expect(SOURCE).toMatch(/audio\.meeting_callout\.title/);
    expect(SOURCE).toMatch(/audio\.meeting_callout\.link/);
    expect(SOURCE).toMatch(/audio\.input_device\.title/);
    expect(SOURCE).toMatch(/audio\.input_device\.microphone\.label/);
    expect(SOURCE).toMatch(/audio\.input_device\.system_default/);
    expect(SOURCE).toMatch(/audio\.input_device\.system_default_caption/);
    expect(SOURCE).toMatch(/audio\.input_device\.live_meter\.label/);
    expect(SOURCE).toMatch(/audio\.input_device\.live_meter\.test/);
    expect(SOURCE).toMatch(/audio\.input_device\.live_meter\.stop/);
    expect(SOURCE).toMatch(/audio\.capture_tail\.title/);
    expect(SOURCE).toMatch(/audio\.capture_tail\.description/);
    expect(SOURCE).toMatch(/audio\.capture_tail\.delay\.label/);
    expect(SOURCE).toMatch(/audio\.capture_tail\.delay\.description/);
    expect(SOURCE).toMatch(/audio\.capture_tail\.delay\.value/);
    expect(SOURCE).toMatch(/audio\.noise_handling\.title/);
    expect(SOURCE).toMatch(/audio\.noise_handling\.suppression\.label/);
    expect(SOURCE).toMatch(/audio\.noise_handling\.suppression\.preview_tooltip/);
    expect(SOURCE).toMatch(/audio\.noise_handling\.suppression\.preview_hint/);
    // Per-level labels and blurbs resolved per-id.
    expect(SOURCE).toMatch(/audio\.noise_handling\.suppression\.levels\.\$\{[a-zA-Z._]*?\}\.label/);
    expect(SOURCE).toMatch(/audio\.noise_handling\.suppression\.levels\.\$\{[a-zA-Z._]*?\}\.blurb/);
    expect(SOURCE).toMatch(/audio\.noise_handling\.noise_gate\.label/);
    expect(SOURCE).toMatch(/audio\.noise_handling\.dynamic_normalisation\.label/);
    expect(SOURCE).toMatch(/audio\.noise_handling\.silence_removal\.label/);
  });

  it('passes the active blurb through the preview tooltip interpolation', () => {
    // The mustache placeholder lives in settings.json; the JSX must
    // pass `{ blurb }` to t() so the catalogue's `{{blurb}}` is
    // substituted at render time. Pin the wiring shape.
    expect(SOURCE).toMatch(
      /audio\.noise_handling\.suppression\.preview_tooltip['"][\s\S]{0,80}blurb/,
    );
  });

  it('keeps backend/OS device names rendering verbatim', () => {
    // The Select option list must continue to render `d.name` as the
    // option label without any t() wrapping — that string is the OS
    // device name and translating it would silently rename the user's
    // microphone in the dropdown. Pin the JSX shape positively, then
    // negative-pin the t-wrapped form to catch a future regression.
    expect(SOURCE).toMatch(/label:\s*d\.name/);
    expect(SOURCE).not.toMatch(/label:\s*t\(['"]audio[^'"]*d\.name/);
    // d.id likewise stays the structural device key.
    expect(SOURCE).toMatch(/value:\s*d\.id/);
  });

  it('preserves the load-bearing IPC commands and audio-test wiring', () => {
    // The structural audio flow is unchanged — every Tauri command
    // name and the meter listen wiring must survive the localisation.
    // Pin each so a future regression that renames a command or
    // breaks the test loop fails immediately.
    const commands = ['list_input_devices', 'start_meter', 'stop_meter', 'show_main_at'];
    for (const cmd of commands) {
      expect(
        SOURCE.includes(cmd),
        `AudioSettings.tsx must keep the IPC command '${cmd}' intact`,
      ).toBe(true);
    }
    // The dictation-meter listen channel and the metering state.
    expect(SOURCE).toMatch(/EVT\.dictationMeter/);
    expect(SOURCE).toMatch(/setMetering\(/);
  });

  it('preserves the underlying audio settings field names', () => {
    // The localisation only swaps display labels. Each control still
    // reads `s.audio.<field>` and writes via `patch({ audio: { ...
    // s.audio, <field>: v } })` against the `audio.<field>` slot on
    // Settings. Pin every load-bearing field name so a future
    // regression that renames a settings field fails immediately.
    const fields = [
      'capture_source',
      'input_device_id',
      'capture_tail_pad_ms',
      'noise_suppression',
      'noise_gate',
      'dynamic_normalisation',
      'silence_removal',
    ];
    for (const field of fields) {
      expect(
        SOURCE.includes(field),
        `AudioSettings.tsx must keep the settings field name '${field}' intact`,
      ).toBe(true);
    }
  });

  it('keeps the end-of-speech slider within the supported capture range', () => {
    expect(SOURCE).toMatch(/min=\{250\}/);
    expect(SOURCE).toMatch(/max=\{1500\}/);
    expect(SOURCE).toMatch(/step=\{250\}/);
  });
});
