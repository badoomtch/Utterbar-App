import { describe, expect, it } from 'vitest';

import { acceleratorToVk } from './hotkey';

describe('acceleratorToVk', () => {
  it('parses single-letter accelerator', () => {
    expect(acceleratorToVk('M')).toBe(0x4d);
    expect(acceleratorToVk('a')).toBe(0x41);
  });

  it('parses Ctrl+letter and ignores modifiers', () => {
    expect(acceleratorToVk('Ctrl+M')).toBe(0x4d);
    expect(acceleratorToVk('Ctrl+Shift+M')).toBe(0x4d);
    expect(acceleratorToVk('Alt+Shift+Win+M')).toBe(0x4d);
  });

  it('parses digits', () => {
    expect(acceleratorToVk('1')).toBe(0x31);
    expect(acceleratorToVk('Ctrl+9')).toBe(0x39);
  });

  it('parses F-keys F1-F24', () => {
    expect(acceleratorToVk('F1')).toBe(0x70);
    expect(acceleratorToVk('F12')).toBe(0x7b);
    expect(acceleratorToVk('F16')).toBe(0x7f);
    expect(acceleratorToVk('F24')).toBe(0x87);
  });

  it('parses named non-printable keys', () => {
    expect(acceleratorToVk('Pause')).toBe(0x13);
    expect(acceleratorToVk('PrintScreen')).toBe(0x2c);
    expect(acceleratorToVk('PrtSc')).toBe(0x2c);
    expect(acceleratorToVk('ScrollLock')).toBe(0x91);
    expect(acceleratorToVk('Insert')).toBe(0x2d);
    expect(acceleratorToVk('Delete')).toBe(0x2e);
    expect(acceleratorToVk('Home')).toBe(0x24);
    expect(acceleratorToVk('End')).toBe(0x23);
    expect(acceleratorToVk('PageUp')).toBe(0x21);
    expect(acceleratorToVk('PageDown')).toBe(0x22);
    expect(acceleratorToVk('Space')).toBe(0x20);
  });

  it('handles whitespace in accelerator parts', () => {
    expect(acceleratorToVk(' Ctrl + Shift + M ')).toBe(0x4d);
  });

  it('returns 0 for unrecognised accelerators', () => {
    expect(acceleratorToVk('')).toBe(0);
    expect(acceleratorToVk('Foo')).toBe(0);
    expect(acceleratorToVk('Ctrl+Foo')).toBe(0);
    // F25 is out of range.
    expect(acceleratorToVk('F25')).toBe(0);
  });

  it('returns 0 for bare modifier-name strings (Shift, Ctrl, etc.)', () => {
    // Shift / Ctrl / Alt / Win as bare strings are not single letters
    // (5+ chars), not digits, not F-keys, not named non-printable
    // keys, so they return 0.
    expect(acceleratorToVk('Shift')).toBe(0);
    expect(acceleratorToVk('Ctrl')).toBe(0);
    expect(acceleratorToVk('Alt')).toBe(0);
  });
});

describe('mic-mute conflict detection', () => {
  // Simulates what MicMuteCard does: derive the VK list from the
  // user's bound shortcuts, then check whether a candidate combo
  // collides.
  function buildConflictVks(accelerators: (string | undefined)[]): number[] {
    return accelerators
      .filter((a): a is string => typeof a === 'string' && a.length > 0)
      .map(acceleratorToVk)
      .filter((vk) => vk !== 0);
  }

  it('flags Toggle Recording = Ctrl+Shift+O conflicting with Ctrl+Shift+O mic-mute', () => {
    const conflicts = buildConflictVks(['Ctrl+Shift+O']);
    expect(conflicts).toContain(0x4f); // 'O'
  });

  it('does not flag mic-mute = F16 against Toggle Recording = Ctrl+Shift+O', () => {
    const conflicts = buildConflictVks(['Ctrl+Shift+O', 'Ctrl+Alt+P']);
    // F16 is 0x7f; not in conflicts.
    expect(conflicts).not.toContain(0x7f);
  });

  it('combines multiple bound shortcuts into one conflict list', () => {
    const conflicts = buildConflictVks([
      'Ctrl+Shift+O',
      'Ctrl+Shift+C',
      'F12',
      undefined,
      '',
    ]);
    expect(conflicts).toEqual(expect.arrayContaining([0x4f, 0x43, 0x7b]));
    expect(conflicts).toHaveLength(3); // undefined and '' filtered out
  });

  it('filters out unparsable shortcuts rather than poisoning the list', () => {
    const conflicts = buildConflictVks(['Ctrl+Shift+O', 'Ctrl+UnknownKey']);
    expect(conflicts).toContain(0x4f);
    expect(conflicts).not.toContain(0); // unparsable returned 0, filtered
  });
});
