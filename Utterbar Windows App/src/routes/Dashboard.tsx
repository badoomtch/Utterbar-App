import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import {
  ArrowUpRight,
  CalendarDays,
  Clock,
  Flame,
  Gauge,
  Hash,
  Mic,
  Sparkles,
  Timer,
  TrendingUp,
} from 'lucide-react';

import { PageHeader } from '@/components/common/PageHeader';
import { MetricCard } from '@/components/common/MetricCard';
import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { formatNumber, longDurationLabel, minutesLabel } from '@/lib/format';

type StatsSummary = {
  words_today: number;
  words_week: number;
  seconds_saved_today: number;
  seconds_saved_week: number;
  avg_wpm: number;
  dictations_today: number;
  dictations_week: number;
  streak_days: number;
  words_all_time: number;
  dictations_all_time: number;
  seconds_saved_all_time: number;
  seconds_spoken_all_time: number;
  days_active_all_time: number;
  first_dictation_day: string | null;
};

type RecentEntry = {
  id: string;
  created_at: number;
  mode_name_snapshot: string;
  processed_text: string;
  raw_text: string;
  audio_duration_ms: number;
  app_exe: string | null;
  app_title: string | null;
  status: string;
};

/**
 * Dashboard — the app's "today" view.
 *
 * Cyan-budget audit (DESIGN_SYSTEM.md §1.6: ≤3 cyan moments per viewport):
 *   1. One "Words today" MetricCard in accent tone (rail + bloom).
 *   2. The sidebar's active nav row rail.
 *   3. (reserved) — a single cyan accent on a callout or CTA if added later.
 *
 * Everything else — recent-activity list, lifetime tiles — lives in
 * neutral + semantic tones only. No cyan text, no cyan fills.
 */
export function Dashboard() {
  const [summary, setSummary] = useState<StatsSummary | null>(null);
  const [recent, setRecent] = useState<RecentEntry[]>([]);

  const loadRecent = () =>
    invoke<RecentEntry[]>('list_history', { filter: { limit: 6 } })
      .then(setRecent)
      .catch(() => setRecent([]));

  useEffect(() => {
    void invoke<StatsSummary>('get_stats_summary').then(setSummary).catch(() => undefined);
    void loadRecent();
  }, []);

  useEffect(() => {
    const p = listen(EVT.dictationResult, () => {
      void invoke<StatsSummary>('get_stats_summary').then(setSummary).catch(() => undefined);
      void loadRecent();
    });
    return () => void p.then((u) => u());
  }, []);

  return (
    <div className="mx-auto max-w-5xl px-10 pt-12 pb-10">
      <PageHeader
        eyebrow="Overview"
        title="Dashboard"
        subtitle="Your local-only dictation stats. Nothing here leaves your machine."
      />

      {/*
        Today / this-week strip. ONE accent tile (the hero number the user
        actually tracks) — rest neutral so the eye has a single anchor.
      */}
      <div className="mt-10 grid grid-cols-2 gap-4 md:grid-cols-4">
        <MetricCard
          label="Words today"
          tone="accent"
          icon={Sparkles}
          value={formatNumber(summary?.words_today ?? 0)}
          caption={`${formatNumber(summary?.dictations_today ?? 0)} dictations`}
        />
        <MetricCard
          label="Words this week"
          tone="neutral"
          icon={TrendingUp}
          value={formatNumber(summary?.words_week ?? 0)}
          caption={`${formatNumber(summary?.dictations_week ?? 0)} dictations`}
        />
        <MetricCard
          label="Time saved today"
          tone="neutral"
          icon={Timer}
          value={minutesLabel(summary?.seconds_saved_today ?? 0)}
          caption="vs. typing baseline"
        />
        <MetricCard
          label="Avg speaking WPM"
          tone="neutral"
          icon={Gauge}
          value={summary ? summary.avg_wpm.toFixed(0) : '—'}
          caption="7-day average"
        />
      </div>

      {/* Lifetime strip — semantic tones only: moss for bragging-rights time,
          amber for the streak flame, neutral for the rest. */}
      <div className="mt-4 grid grid-cols-2 gap-4 md:grid-cols-4">
        <MetricCard
          label="Words all time"
          tone="neutral"
          icon={Hash}
          value={formatNumber(summary?.words_all_time ?? 0)}
          caption={`${formatNumber(summary?.dictations_all_time ?? 0)} dictations${
            summary?.first_dictation_day ? ` · since ${formatDay(summary.first_dictation_day)}` : ''
          }`}
        />
        <MetricCard
          label="Time saved all time"
          tone="positive"
          icon={Clock}
          value={longDurationLabel(summary?.seconds_saved_all_time ?? 0)}
          caption="vs. typing baseline"
        />
        <MetricCard
          label="Active days"
          tone="warning"
          icon={Flame}
          value={formatNumber(summary?.days_active_all_time ?? 0)}
          caption={streakCaption(summary?.streak_days ?? 0)}
        />
        <MetricCard
          label="Time spoken"
          tone="neutral"
          icon={CalendarDays}
          value={longDurationLabel(summary?.seconds_spoken_all_time ?? 0)}
          caption="total time the mic was on"
        />
      </div>

      {/* Timeline card — bg-1, no shadow, no border. Surface tone is the
          separation per §4.2. */}
      <section className="mt-12 rounded-card bg-bg-1 p-6">
        <div className="flex items-center justify-between gap-4">
          <div>
            <div className="ub-eyebrow mb-1.5">Timeline</div>
            {/* Weight 700 (H3 tier per §2.3). font-semibold (600) forbidden. */}
            <h2 className="text-[18px] font-bold text-fg-hi -tracking-[0.02em] leading-tight">
              Recent activity
            </h2>
            <p className="mt-1.5 text-[13px] text-muted leading-relaxed">
              Your last few dictations. Click through to History for the full timeline.
            </p>
          </div>
          <Link
            to="/history"
            className="inline-flex items-center gap-1.5 rounded-input bg-bg-2 px-3 py-1.5 text-[12px] text-muted transition-colors hover:bg-bg-2/80 hover:text-fg-hi"
          >
            Open History <ArrowUpRight className="h-3 w-3" />
          </Link>
        </div>

        {recent.length === 0 ? (
          <div className="mt-8 flex flex-col items-center gap-2 py-8 text-center text-[12.5px] text-fg-dim">
            <Mic className="h-6 w-6 opacity-60" />
            <span>Dictate anywhere with your shortcut and your entries will land here.</span>
          </div>
        ) : (
          <ul className="mt-5 divide-y divide-border-soft/50">
            {recent.map((e) => (
              <RecentRow key={e.id} entry={e} />
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}

function RecentRow({ entry }: { entry: RecentEntry }) {
  const text = (entry.processed_text || entry.raw_text || '').replace(/\s+/g, ' ').trim();
  const preview = text.length > 140 ? `${text.slice(0, 140)}…` : text || '(empty)';
  const app = entry.app_title || entry.app_exe || 'Unknown app';
  return (
    <li className="py-3.5">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="truncate text-[13px] text-fg-hi">{preview}</div>
          <div className="mt-1 flex items-center gap-2 text-[11.5px] text-fg-dim">
            <span className="truncate">{app}</span>
            <span className="text-fg-dim/60">·</span>
            <span>{entry.mode_name_snapshot}</span>
            <span className="text-fg-dim/60">·</span>
            <span className="font-mono tabular-nums">{relativeTime(entry.created_at)}</span>
          </div>
        </div>
      </div>
    </li>
  );
}

function streakCaption(streakDays: number): string {
  if (streakDays <= 0) return 'dictate today to start a streak';
  if (streakDays === 1) return 'current streak: 1 day';
  return `current streak: ${streakDays} days`;
}

function formatDay(ymd: string): string {
  const d = new Date(`${ymd}T12:00:00Z`);
  if (Number.isNaN(d.getTime())) return ymd;
  return d.toLocaleDateString(undefined, { day: 'numeric', month: 'short', year: 'numeric' });
}

function relativeTime(createdAtMs: number): string {
  const delta = Date.now() - createdAtMs;
  if (delta < 60_000) return 'just now';
  if (delta < 3_600_000) return `${Math.floor(delta / 60_000)} min ago`;
  if (delta < 86_400_000) return `${Math.floor(delta / 3_600_000)}h ago`;
  const days = Math.floor(delta / 86_400_000);
  if (days < 7) return `${days}d ago`;
  return new Date(createdAtMs).toLocaleDateString();
}
