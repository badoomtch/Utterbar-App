export function formatDuration(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const m = Math.floor(totalSeconds / 60);
  const s = totalSeconds % 60;
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
}

export function formatNumber(n: number): string {
  return n.toLocaleString(undefined);
}

export function formatBytes(bytes: number): string {
  if (!bytes) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  let i = 0;
  let v = bytes;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i += 1;
  }
  return `${v.toFixed(v >= 10 || i === 0 ? 0 : 1)} ${units[i]}`;
}

export function wordsSaved(words: number, spokenSecs: number, baselineWpm: number): number {
  if (baselineWpm <= 0) return 0;
  const typingSecs = (words * 60) / baselineWpm;
  return Math.max(0, typingSecs - spokenSecs);
}

export function minutesLabel(seconds: number): string {
  const m = Math.round(seconds / 60);
  if (m < 1) return '< 1 min';
  if (m < 60) return `${m} min`;
  const h = Math.floor(m / 60);
  const rem = m % 60;
  return `${h}h ${rem}m`;
}

/**
 * Longer-horizon duration formatter used for all-time rollups. Once the total
 * crosses ~2 days the hours-only format becomes unreadable, so we switch to
 * `Xd Yh`. Under a day we still say `Xh Ym` so the number the user sees on
 * their first active day doesn't suddenly change shape the moment they cross
 * 24h.
 */
export function longDurationLabel(seconds: number): string {
  if (seconds < 60) return '< 1 min';
  const totalMinutes = Math.round(seconds / 60);
  if (totalMinutes < 60) return `${totalMinutes} min`;
  const totalHours = Math.floor(totalMinutes / 60);
  if (totalHours < 48) {
    const remMinutes = totalMinutes % 60;
    return `${totalHours}h ${remMinutes}m`;
  }
  const days = Math.floor(totalHours / 24);
  const remHours = totalHours % 24;
  return `${days}d ${remHours}h`;
}
