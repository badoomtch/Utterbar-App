/**
 * Hotkey accelerator parsing helpers.
 *
 * Used by the experimental Teams + Discord mic-mute Settings card to
 * extract VK codes from Utterbar's own global-shortcut accelerator
 * strings (`"Ctrl+Shift+O"`, `"F12"`, etc.) so the user can't
 * accidentally bind a mic-mute hotkey that would collide with
 * Toggle Recording / Cancel / Cycle Mode etc.
 *
 * Parser is forgiving by design: an unrecognised accelerator yields
 * 0, which the caller filters out. We never want a parse failure to
 * block hotkey capture; the worst case of a missed conflict is the
 * user trying their hotkey and having it fight Utterbar, which they
 * can fix by re-recording.
 */

/**
 * Parse a Utterbar shortcut accelerator string and return the Win32
 * VK code of its main key. Returns 0 if the accelerator can't be
 * parsed (e.g. uses a named key we don't map).
 */
export function acceleratorToVk(accelerator: string): number {
  const parts = accelerator
    .split('+')
    .map((p) => p.trim())
    .filter(Boolean);
  if (parts.length === 0) return 0;
  const mainKey = parts[parts.length - 1];
  if (/^[a-zA-Z]$/.test(mainKey)) {
    return mainKey.toUpperCase().charCodeAt(0);
  }
  if (/^[0-9]$/.test(mainKey)) {
    return mainKey.charCodeAt(0);
  }
  const fMatch = /^F(\d{1,2})$/i.exec(mainKey);
  if (fMatch) {
    const n = Number(fMatch[1]);
    if (n >= 1 && n <= 24) {
      return 0x70 + (n - 1);
    }
  }
  switch (mainKey.toLowerCase()) {
    case 'pause':
      return 0x13;
    case 'printscreen':
    case 'prtsc':
      return 0x2c;
    case 'scrolllock':
    case 'scroll':
      return 0x91;
    case 'insert':
    case 'ins':
      return 0x2d;
    case 'delete':
    case 'del':
      return 0x2e;
    case 'home':
      return 0x24;
    case 'end':
      return 0x23;
    case 'pageup':
      return 0x21;
    case 'pagedown':
      return 0x22;
    case 'space':
      return 0x20;
    default:
      return 0;
  }
}
