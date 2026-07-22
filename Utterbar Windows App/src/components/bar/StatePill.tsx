import { motion, AnimatePresence } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import type { TFunction } from 'i18next';
import { cn } from '@/lib/cn';
import { springs } from '@/design/motion';
import { useDictation, type DictationPhase, type MeetingProgress } from '@/state/dictation';

/**
 * StatePill — the dictation-phase read inside the floating bar.
 *
 * Per DESIGN_SYSTEM.md §8 the pill is the signature surface. The state
 * label rides on it as a quiet chip:
 *
 *   • Idle / warming / cancelled — neutral dot + muted text on bg-2.
 *   • Recording / processing     — cyan LED (pulse for recording) + accent
 *                                  text. The cyan-count budget allows this
 *                                  because the rest of the bar is dark.
 *   • Inserted (success)         — moss dot + moss text.
 *   • Error                      — rose dot + rose text.
 *
 * The label uses Geist Mono (§2.4: mono reserved for timestamps, hash IDs,
 * and state reads — exactly this).
 */
export function StatePill({ phase }: { phase: DictationPhase }) {
  // Read meeting progress directly from the store rather than threading
  // it through props from FloatingBar. The store slot is null for every
  // phase except `transcribing_background`, and `setPhase` clears it on
  // every transition out of that phase, so reading it unconditionally
  // here is safe (and a re-render is only triggered when the value
  // changes, by zustand's default reference equality).
  const meetingProgress = useDictation((s) => s.meetingProgress);
  const { t } = useTranslation('bar');
  const { label, tone } = describe(phase, meetingProgress, t);
  // The progress track is a thin accent line under the pill that fills
  // proportionally as chunks complete. Only shown for the meeting-
  // transcribe phase WITH a non-null progress payload (single-chunk
  // fast path doesn't emit, so this never appears for short meetings).
  const showProgress =
    phase.kind === 'transcribing_background' &&
    meetingProgress !== null &&
    meetingProgress.totalChunks > 0;
  // Each chunk emits its progress at the START of its work, so showing
  // (chunkIndex + 1) / totalChunks reads as "we are now on chunk N of
  // M" rather than "N chunks have completed". Clamp to [0, 1] so a
  // late event with chunk_index >= total_chunks (shouldn't happen but
  // be defensive) still renders sanely.
  const fraction = showProgress
    ? Math.min(
        1,
        Math.max(0, (meetingProgress.chunkIndex + 1) / meetingProgress.totalChunks),
      )
    : 0;
  return (
    <AnimatePresence mode="popLayout" initial={false}>
      <motion.span
        key={phase.kind}
        layout
        initial={{ opacity: 0, y: 4, scale: 0.96 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: -4, scale: 0.96 }}
        transition={springs.pop}
        className={cn(
          // Geist Mono. Bumped from 9.5px to 11px — the old size read as
          // decorative ornament; 'Ready' was barely legible on the floating
          // bar's gradient, which defeated the whole point of surfacing the
          // idle state. 11px + 0.08em tracking keeps the chip compact while
          // actually being readable at a glance.
          'relative inline-flex cursor-default select-none items-center gap-1.5 rounded-full px-2 py-0.5',
          'font-mono text-[11px] font-medium uppercase tracking-[0.08em]',
          toneClass(tone),
        )}
      >
        <span className={cn('h-1.5 w-1.5 rounded-full', dotClass(tone))} />
        {label}
        {showProgress && (
          <span
            aria-hidden
            className="pointer-events-none absolute inset-x-2 bottom-0.5 h-[1.5px] overflow-hidden rounded-full bg-[color-mix(in_oklab,var(--accent)_18%,transparent)]"
          >
            <span
              className="block h-full origin-left bg-accent transition-[width] duration-300 ease-out"
              style={{ width: `${(fraction * 100).toFixed(1)}%` }}
            />
          </span>
        )}
      </motion.span>
    </AnimatePresence>
  );
}

function describe(
  phase: DictationPhase,
  meetingProgress: MeetingProgress | null,
  t: TFunction<'bar'>,
) {
  switch (phase.kind) {
    case 'idle':
      return { label: t('phase.ready'), tone: 'muted' as const };
    case 'warming':
      // v0.6.7: `Warming` was the previous copy but read as too generic —
      // users saw `Processing` then `Warming` and assumed both meant the
      // same "thinking" state. The Rust side now only emits this phase on a
      // genuine cold-load, so the copy explicitly says so and uses the
      // accent tone (not muted) to signal "something is happening, just
      // longer than usual" rather than "idle".
      return { label: t('phase.initialising_model'), tone: 'accent' as const };
    case 'listening':
      return { label: t('phase.recording'), tone: 'accent' as const };
    case 'processing':
      return { label: t('phase.processing'), tone: 'accent' as const };
    case 'transcribing_background': {
      // Meeting mode's chunked transcribe pass. Same visual treatment as
      // Processing (accent tone, same dot) but the copy tells the user
      // what's actually happening instead of the generic "Processing" that
      // for a 5-min meeting can sit on screen for ~17 s with no context.
      //
      // Append a "N/M" counter when chunk progress is known so the user
      // can read concrete progress at a glance instead of a static
      // ellipsis. Falls back to the bare phrase for the single-chunk fast
      // path (which does not emit progress) and for the brief moment
      // between phase change and first progress event arrival.
      if (meetingProgress && meetingProgress.totalChunks > 0) {
        const current = meetingProgress.chunkIndex + 1;
        return {
          label: t('phase.transcribing_meeting_with_progress', {
            current,
            total: meetingProgress.totalChunks,
          }),
          tone: 'accent' as const,
        };
      }
      return { label: t('phase.transcribing_meeting'), tone: 'accent' as const };
    }
    case 'success':
      // Meeting-mode success carries a summary so the user gets concrete
      // feedback ("5:01 · 11 chunks · 799 chars") instead of the generic
      // success label that was indistinguishable between a 3-second
      // dictation and a 5-minute meeting that happened to process fast.
      // Ordinary dictations don't carry a summary and keep the original
      // copy.
      if (phase.summary) {
        const { duration_seconds, chunks, chars, model_label } = phase.summary;
        const mm = Math.floor(duration_seconds / 60);
        const ss = duration_seconds % 60;
        const time = `${mm}:${ss.toString().padStart(2, '0')}`;
        // v0.15.3: model_label slots between time and chunk count so
        // the user can correlate quality vs speed across meetings.
        // Older events without the field would render `undefined` —
        // use a defensive fallback.
        const modelSegment = model_label ? ` · ${model_label}` : '';
        const chunkWord = t(chunks === 1 ? 'unit.chunk_one' : 'unit.chunk_other');
        const label = t('phase.saved', {
          time,
          modelSegment,
          chunks,
          chunkWord,
          chars: chars.toLocaleString(),
        });
        return { label, tone: 'success' as const };
      }
      // When the Rust auto-stop timer drove this stop (single-shot
      // dictation hit the 5-minute cap), surface a clear non-error
      // label instead of the generic success copy so the user
      // understands why their recording ended without their hotkey
      // press. Audio up to the cap was transcribed and inserted
      // normally.
      if (phase.notice) {
        return { label: t('phase.auto_stopped_5_min'), tone: 'success' as const };
      }
      return { label: t('phase.inserted'), tone: 'success' as const };
    case 'error':
      return { label: phase.message || t('phase.error'), tone: 'error' as const };
    case 'cancelled':
      return { label: t('phase.cancelled'), tone: 'muted' as const };
    case 'no_audio':
      // Muted tone — silence is a non-event, not a failure. The user
      // tapped the hotkey but nothing usable came out; we want the
      // message legible, not alarming.
      //
      // `phase.notice` is set by the Rust output validator when the
      // drop wasn't plain silence (e.g. a decoder loop or a foreign-
      // script hallucination on an English-only path). We surface it
      // verbatim, since it is dynamic Rust-emitted prose rather than a
      // fixed UI string. Localising backend messages is a later branch.
      if (phase.notice) {
        return { label: phase.notice, tone: 'muted' as const };
      }
      return { label: t('phase.no_audio_detected'), tone: 'muted' as const };
  }
}

// Tone classes. StatePill only appears inside the floating bar, so every
// tone mixes its accent colour with the pill-local surface (`--pill-surface`)
// rather than the app-shell `--bg-2`. That was the 'Ready' bug on Paper: a
// dark-grey chip on a cream pill looked like a paste error. Mixing against
// `--pill-surface` flips correctly per theme (dark tile on dark pill, warm
// tile on Paper's light pill) and keeps the chip recessed into the bar
// instead of floating on top.
function toneClass(tone: 'muted' | 'accent' | 'success' | 'error') {
  switch (tone) {
    case 'accent':
      return 'bg-[color-mix(in_oklab,var(--accent)_16%,var(--pill-surface))] text-accent';
    case 'success':
      return 'bg-[color-mix(in_oklab,var(--moss)_16%,var(--pill-surface))] text-moss';
    case 'error':
      return 'bg-[color-mix(in_oklab,var(--rose)_16%,var(--pill-surface))] text-rose';
    default:
      // Muted 'Ready' uses the pill-local surface + muted ink tokens so it
      // reads cleanly on both dark and Paper pills without hand-tuning.
      return 'bg-[var(--pill-surface)] text-[var(--pill-fg-muted)]';
  }
}

function dotClass(tone: 'muted' | 'accent' | 'success' | 'error') {
  switch (tone) {
    case 'accent':
      return 'bg-accent animate-pulseDot';
    case 'success':
      return 'bg-moss';
    case 'error':
      return 'bg-rose';
    default:
      // Matches the muted-text colour of the surrounding chip so the dot
      // doesn't out-contrast the label on Paper (where `fg-dim` is the
      // app-shell token, not the pill-local one).
      return 'bg-[var(--pill-fg-muted)]';
  }
}
