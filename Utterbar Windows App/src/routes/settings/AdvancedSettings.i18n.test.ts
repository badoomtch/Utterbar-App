import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'AdvancedSettings.tsx'), 'utf8');

// Strip line + block comments before grepping so prose commentary in
// AdvancedSettings.tsx (which legitimately mentions things like
// "DirectML", "Whisper Small", or "Reset") doesn't
// false-trigger the guard.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('AdvancedSettings i18n source-grep guard', () => {
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
      'title="Advanced"',
      'subtitle="Diagnostics, hardware preferences, and experimental toggles."',
      // GPU card title.
      'title="GPU acceleration"',
      // GPU_LABELS dictionary entries — pre-localisation per-id title
      // and hint pairs that lived inline in the const at module scope.
      "title: 'Auto'",
      "title: 'DirectML'",
      "title: 'CPU only'",
      "hint: 'Use your GPU if one is detected",
      "hint: 'Prefer DirectML",
      "hint: 'Keep the GPU free for games or video.",
      // The four pre-localisation card description literals.
      "'Checking what hardware is available…'",
      "'This build runs on CPU only.",
      "`Detected ${bestAdapter.name}.",
      "'GPU acceleration is built in but no dedicated",
      // Disabled / inactive notice literals.
      "', not available on this machine.'",
      "Selected, but currently inactive — this build does not include",
      // Detected adapters eyebrow + per-adapter suffixes.
      '>Detected adapters<',
      "' (software)'",
      ", ${a.dedicated_video_memory_mb.toLocaleString()} MB VRAM",
      // Diagnostics card props + per-row labels.
      'title="Diagnostics"',
      'description="Everything below is read from this machine.',
      '>Version<',
      '>OS<',
      '>CPU threads<',
      '>ONNX backend<',
      '>llama.cpp backend<',
      '>Logs<',
      '>Database<',
      '>Models<',
      "? 'compiled' : 'disabled'",
      // Backup & restore card props + row labels/descriptions.
      'title="Backup & restore"',
      'label="Include recordings in export"',
      'label="Export data"',
      'label="Import data"',
      "'Writes a .tar file to the location you choose.'",
      "Last export: ${formatBytes(",
      "? 'Exporting…' : 'Export…'",
      "? 'Importing…' : 'Import…'",
      // Save / open dialog literals.
      "title: 'Export Utterbar data'",
      "title: 'Import Utterbar backup'",
      "name: 'Utterbar backup'",
      // Reset card.
      'title="Reset"',
      'label="Reset all settings to defaults"',
      "'Reset all Utterbar settings? This does not delete recordings or history.'",
      '>Reset<',
      // Import-confirmation modal literals.
      '>Destructive action<',
      '>Replace all Utterbar data?<',
      'Importing <span',
      'will overwrite your current settings, modes, vocabulary',
      'This cannot be undone. If you',
      '>Cancel<',
      "? 'Importing & restarting…' : 'Replace and restart'",
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `AdvancedSettings.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('routes the page header and visible chrome through advanced.* keys', () => {
    expect(SOURCE).toMatch(/advanced\.title/);
    expect(SOURCE).toMatch(/advanced\.subtitle/);
    // GPU card.
    expect(SOURCE).toMatch(/advanced\.gpu\.title/);
    expect(SOURCE).toMatch(/advanced\.gpu\.description\.checking/);
    expect(SOURCE).toMatch(/advanced\.gpu\.description\.no_runtime_ep/);
    expect(SOURCE).toMatch(/advanced\.gpu\.description\.detected/);
    expect(SOURCE).toMatch(/advanced\.gpu\.description\.no_gpu/);
    // Per-option title and hint resolved per-id at render time.
    expect(SOURCE).toMatch(/advanced\.gpu\.options\.\$\{[^}]*?\}\.title/);
    expect(SOURCE).toMatch(/advanced\.gpu\.options\.\$\{[^}]*?\}\.hint/);
    expect(SOURCE).toMatch(/advanced\.gpu\.disabled_suffix/);
    expect(SOURCE).toMatch(/advanced\.gpu\.inactive_notice/);
    expect(SOURCE).toMatch(/advanced\.gpu\.detected_adapters/);
    expect(SOURCE).toMatch(/advanced\.gpu\.adapter_software_suffix/);
    expect(SOURCE).toMatch(/advanced\.gpu\.adapter_vram_suffix/);
    // Diagnostics card.
    expect(SOURCE).toMatch(/advanced\.diagnostics\.title/);
    expect(SOURCE).toMatch(/advanced\.diagnostics\.description/);
    expect(SOURCE).toMatch(/advanced\.diagnostics\.rows\.version/);
    expect(SOURCE).toMatch(/advanced\.diagnostics\.rows\.os/);
    expect(SOURCE).toMatch(/advanced\.diagnostics\.rows\.cpu_threads/);
    expect(SOURCE).toMatch(/advanced\.diagnostics\.rows\.onnx/);
    expect(SOURCE).toMatch(/advanced\.diagnostics\.rows\.llama/);
    expect(SOURCE).toMatch(/advanced\.diagnostics\.rows\.logs/);
    expect(SOURCE).toMatch(/advanced\.diagnostics\.rows\.database/);
    expect(SOURCE).toMatch(/advanced\.diagnostics\.rows\.models/);
    expect(SOURCE).toMatch(/advanced\.diagnostics\.feature_state\.compiled/);
    expect(SOURCE).toMatch(/advanced\.diagnostics\.feature_state\.disabled/);
    // Backup card.
    expect(SOURCE).toMatch(/advanced\.backup\.title/);
    expect(SOURCE).toMatch(/advanced\.backup\.description/);
    expect(SOURCE).toMatch(/advanced\.backup\.include_recordings\.label/);
    expect(SOURCE).toMatch(/advanced\.backup\.export\.label/);
    expect(SOURCE).toMatch(/advanced\.backup\.export\.description_idle/);
    expect(SOURCE).toMatch(/advanced\.backup\.export\.description_last/);
    expect(SOURCE).toMatch(/advanced\.backup\.export\.contents_nothing/);
    expect(SOURCE).toMatch(/advanced\.backup\.export\.exporting/);
    expect(SOURCE).toMatch(/advanced\.backup\.export\.button/);
    expect(SOURCE).toMatch(/advanced\.backup\.export\.dialog_title/);
    expect(SOURCE).toMatch(/advanced\.backup\.export\.filter_name/);
    expect(SOURCE).toMatch(/advanced\.backup\.import\.label/);
    expect(SOURCE).toMatch(/advanced\.backup\.import\.description/);
    expect(SOURCE).toMatch(/advanced\.backup\.import\.importing/);
    expect(SOURCE).toMatch(/advanced\.backup\.import\.button/);
    expect(SOURCE).toMatch(/advanced\.backup\.import\.dialog_title/);
    // Modal.
    expect(SOURCE).toMatch(/advanced\.backup\.import_modal\.eyebrow/);
    expect(SOURCE).toMatch(/advanced\.backup\.import_modal\.heading/);
    expect(SOURCE).toMatch(/advanced\.backup\.import_modal\.body_prefix/);
    expect(SOURCE).toMatch(/advanced\.backup\.import_modal\.body_suffix/);
    expect(SOURCE).toMatch(/advanced\.backup\.import_modal\.warning/);
    expect(SOURCE).toMatch(/advanced\.backup\.import_modal\.cancel/);
    expect(SOURCE).toMatch(/advanced\.backup\.import_modal\.replace/);
    expect(SOURCE).toMatch(/advanced\.backup\.import_modal\.replacing/);
    // Reset card.
    expect(SOURCE).toMatch(/advanced\.reset\.title/);
    expect(SOURCE).toMatch(/advanced\.reset\.row\.label/);
    expect(SOURCE).toMatch(/advanced\.reset\.row\.description/);
    expect(SOURCE).toMatch(/advanced\.reset\.button/);
    expect(SOURCE).toMatch(/advanced\.reset\.confirm/);
  });

  it('passes interpolation values through t() at the call sites', () => {
    // Pin every interpolated key + the variable-name shape it expects.
    // The detected-GPU description carries `name`; the VRAM suffix
    // carries `mb`; the last-export description carries `size` and
    // `contents`. A future regression that drops any argument fails
    // at build time, not by leaking a literal mustache template to
    // users.
    expect(SOURCE).toMatch(
      /advanced\.gpu\.description\.detected['"][\s\S]{0,80}name:\s*bestAdapter\.name/,
    );
    expect(SOURCE).toMatch(
      /advanced\.gpu\.adapter_vram_suffix['"][\s\S]{0,120}mb:\s*a\.dedicated_video_memory_mb/,
    );
    expect(SOURCE).toMatch(
      /advanced\.backup\.export\.description_last['"][\s\S]{0,200}size:\s*formatBytes/,
    );
    expect(SOURCE).toMatch(
      /advanced\.backup\.export\.description_last['"][\s\S]{0,200}contents:/,
    );
  });

  it('keeps backend-emitted Diagnostics, GPU, and error values verbatim', () => {
    // Every <dd> in the diagnostics dl renders a backend-emitted
    // string. The OS/driver-emitted GPU adapter name and VRAM
    // figure flow through verbatim. The Rust-side backupError is
    // rendered untranslated. The basename split off the pending
    // import path (path.split(/[\\/]/).pop()) is also a backend-
    // emitted filename. Pin every shape positively, plus a
    // negative pin against t()-wrapped versions.
    expect(SOURCE).toMatch(/\{diag\.version\}/);
    expect(SOURCE).toMatch(/\{diag\.os\}/);
    expect(SOURCE).toMatch(/\{diag\.cpu_count\}/);
    expect(SOURCE).toMatch(/\{diag\.log_path\}/);
    expect(SOURCE).toMatch(/\{diag\.db_path\}/);
    expect(SOURCE).toMatch(/\{diag\.models_path\}/);
    expect(SOURCE).toMatch(/\{a\.name\}/);
    expect(SOURCE).toMatch(/\{backupError\}/);
    // The path-basename split uses a regex literal that's awkward to
    // re-encode inside another regex; pin via substring instead.
    expect(SOURCE.includes('pendingImportPath.split(')).toBe(true);
    expect(SOURCE.includes('.pop()')).toBe(true);
    // Negative pins: none of the diag fields end up wrapped in a
    // catalogue path.
    expect(SOURCE).not.toMatch(/t\(['"]advanced[^'"]*\{diag\.version\}/);
    expect(SOURCE).not.toMatch(/t\(['"]advanced[^'"]*\{backupError\}/);
  });

  it('preserves the load-bearing IPC commands and dialog plugin wiring', () => {
    // The structural diagnostics + backup/restore + reset flow is
    // unchanged — every Tauri command name and the dialog plugin
    // entry points must survive the localisation. Pin each so a
    // future regression that renames a command or breaks the
    // export / import / reset round-trip fails immediately.
    const commands = [
      'get_diagnostics',
      'probe_gpu',
      'runtime_capabilities',
      'open_path',
      'export_user_data',
      'import_user_data',
    ];
    for (const cmd of commands) {
      expect(
        SOURCE.includes(cmd),
        `AdvancedSettings.tsx must keep the IPC command '${cmd}' intact`,
      ).toBe(true);
    }
    // Tauri dialog plugin entry points used for export / import.
    expect(SOURCE).toMatch(/saveDialog\(/);
    expect(SOURCE).toMatch(/openDialog\(/);
    // Settings-store reset hook the destructive button calls.
    expect(SOURCE).toMatch(/useSettings\(\(x\)\s*=>\s*x\.reset\)/);
  });

  it('preserves the underlying settings field name and channel ids', () => {
    // The localisation only swaps display labels. The patch wiring
    // still writes to `s.advanced.gpu_acceleration`, and the
    // GpuAcceleration enum string ids must stay structural.
    expect(
      SOURCE.includes('gpu_acceleration'),
      "AdvancedSettings.tsx must keep the settings field name 'gpu_acceleration' intact",
    ).toBe(true);
    expect(SOURCE).toMatch(
      /GPU_OPTION_IDS:\s*GpuAcceleration\[\]\s*=\s*\[\s*'auto',\s*'direct_ml',\s*'cpu'\s*\]/,
    );
  });
});
