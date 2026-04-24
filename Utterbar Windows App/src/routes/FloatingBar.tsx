import { motion, AnimatePresence } from 'framer-motion';
import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import {
  Check,
  CircleStop,
  Copy as CopyIcon,
  Settings as SettingsIcon,
  SquarePlay,
  X,
  type LucideIcon,
} from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { StatePill } from '@/components/bar/StatePill';
import { UtterbarMiniMark } from '@/components/bar/UtterbarMiniMark';
import { EVT } from '@/lib/events';
import { invoke, listen } from '@/lib/ipc';
import { cn } from '@/lib/cn';
import { formatDuration } from '@/lib/format';
import { useDictation } from '@/state/dictation';
import { useSettings } from '@/state/settings';

export function FloatingBar() {
  const phase = useDictation((s) => s.phase);
  // Last dictation text kept in the zustand store — populated from the
  // `dictation:result` event in AppBootstrap. Null until the user dictates
  // once in the current session; we disable the Copy tile in that case.
  const lastText = useDictation((s) => s.lastText);
  // Meter readings are no longer consumed at this layer — the prototype v2
  // wave bars are CSS-keyframe driven (see `.ub-bar-wave__bar` in globals),
  // which gives a steadier "I'm listening" cue than amplitude-bound bars
  // (RMS goes near-zero between syllables and the wave would judder). If we
  // want meter-reactive bars back later, hook `useDictation(s => s.meter)`
  // here and drive the bar heights via CSS var on the container.
  const elapsed = useDictation((s) => s.elapsedMs);
  const tick = useDictation((s) => s.tickElapsed);

  const [hovered, setHovered] = useState(false);
  // Drag-lift signal — the prototype's tactile "picks up off the desktop"
  // cue. True for the duration of a pointer-down-drag on the pill; triggers
  // a scale-up, accent border, and keeps the bar expanded so the user can
  // see what they're dragging. Cleared on any global pointerup so the lift
  // can't get stuck if the user releases outside the window (which does
  // happen when dragging to a screen edge).
  const [dragging, setDragging] = useState(false);
  // Busy-flash signal — set true for 900ms when Rust fires the
  // `dictation:busy` event. This happens when the user pressed the
  // hotkey while the app was mid-transcribe / mid-recording. We flash
  // the accent border for a moment so the user gets a visible "noted,
  // try again" cue instead of the hotkey silently doing nothing.
  const [busyFlash, setBusyFlash] = useState(false);
  const busyFlashTimerRef = useRef<number | undefined>(undefined);
  // Drag-termination detection.
  //
  // Before v0.6.4 this used a global `pointerup` listener to clear
  // `dragging`. That was wrong: Tauri's `startDragging()` hands the
  // window move to the OS immediately, and the OS's window-move loop
  // synthesises a pointerup on the webview the moment it takes over —
  // which fired our clear immediately after drag-start, killing the
  // accent border within a frame. The user saw a flash, not a
  // sustained highlight.
  //
  // The reliable "drag ended" signal is a **silence window** after
  // `onMoved` events stop arriving from the Tauri window. We debounce:
  // every onMoved re-arms a 220 ms timer, and when the timer fires
  // (no more moves for 220 ms) we clear `dragging`. If the user drags
  // continuously, `dragging` stays true. If they release, moves stop
  // within a frame and `dragging` clears ~220 ms later — smooth,
  // deterministic, and never clears mid-drag.
  //
  // Safety nets:
  //   • A hard fallback timer (2.5 s) clears `dragging` even if onMoved
  //     never fires — some window managers don't emit moves for a zero-
  //     distance click-and-hold, so a stuck drag-lift is possible
  //     without this floor.
  //   • Window `blur` still clears — if the OS moves focus away during
  //     a drag something's wrong and we'd rather reset than stick.
  useEffect(() => {
    if (!dragging) return;
    let moveTimer: number | undefined;
    let hardFallback: number | undefined;
    let unlistenMoved: (() => void) | undefined;
    let cancelled = false;

    const scheduleClear = () => {
      if (moveTimer !== undefined) window.clearTimeout(moveTimer);
      moveTimer = window.setTimeout(() => {
        if (!cancelled) setDragging(false);
      }, 220);
    };

    // Initial timer so zero-move click-and-hold eventually releases.
    scheduleClear();
    hardFallback = window.setTimeout(() => {
      if (!cancelled) setDragging(false);
    }, 2500);

    void getCurrentWindow()
      .onMoved(() => {
        scheduleClear();
      })
      .then((u) => {
        if (cancelled) u();
        else unlistenMoved = u;
      })
      .catch(() => undefined);

    const onBlur = () => setDragging(false);
    window.addEventListener('blur', onBlur);

    return () => {
      cancelled = true;
      if (moveTimer !== undefined) window.clearTimeout(moveTimer);
      if (hardFallback !== undefined) window.clearTimeout(hardFallback);
      unlistenMoved?.();
      window.removeEventListener('blur', onBlur);
    };
  }, [dragging]);
  const pillRef = useRef<HTMLDivElement>(null);

  // Subscribe to the `dictation:busy` event Rust emits when a hotkey
  // press was rejected because the app was already recording or
  // transcribing. We toggle `busyFlash` on for 900ms so the accent
  // border lights briefly — user sees "noted, but not yet" instead
  // of a silent no-op.
  useEffect(() => {
    const p = listen(EVT.dictationBusy, () => {
      setBusyFlash(true);
      if (busyFlashTimerRef.current !== undefined) {
        window.clearTimeout(busyFlashTimerRef.current);
      }
      busyFlashTimerRef.current = window.setTimeout(() => {
        setBusyFlash(false);
        busyFlashTimerRef.current = undefined;
      }, 900);
    });
    return () => {
      void p.then((u) => u()).catch(() => undefined);
      if (busyFlashTimerRef.current !== undefined) {
        window.clearTimeout(busyFlashTimerRef.current);
      }
    };
  }, []);

  // Hover-off grace period. The "it snaps closed as you leave" problem has
  // two sources:
  //   1. Drags: the OS drag immediately takes the cursor off the pill, which
  //      fires mouseleave and collapses the bar mid-drag. We already force
  //      `expanded` on while dragging, but we also want the post-drop hover
  //      to stick around for a beat rather than snapping closed.
  //   2. Quick mouse-outs: on smaller pointer movements over the edge, the
  //      bar would snap shut before the user had a chance to click a button.
  // A 220 ms decay gives the bar a gentle close animation when you leave it,
  // and lets the layout spring do its job on the way back in.
  const leaveTimerRef = useRef<number | undefined>(undefined);
  const setHoveredDelayed = (v: boolean) => {
    if (v) {
      if (leaveTimerRef.current !== undefined) {
        window.clearTimeout(leaveTimerRef.current);
        leaveTimerRef.current = undefined;
      }
      setHovered(true);
      return;
    }
    if (leaveTimerRef.current !== undefined) return;
    leaveTimerRef.current = window.setTimeout(() => {
      leaveTimerRef.current = undefined;
      // Final truth check before committing — if the real :hover is still
      // on the pill (cursor came back), don't collapse.
      const el = pillRef.current;
      if (el && el.matches(':hover')) return;
      setHovered(false);
    }, 220);
  };
  useEffect(() => {
    return () => {
      if (leaveTimerRef.current !== undefined) window.clearTimeout(leaveTimerRef.current);
    };
  }, []);

  // Guard against stuck-open: onMouseLeave sometimes doesn't fire when the cursor
  // whips off the pill too fast, or when the window loses pointer focus mid-move. Poll
  // the real CSS `:hover` state while we *think* we're hovered and correct ourselves.
  useEffect(() => {
    if (!hovered) return;
    const check = () => {
      const el = pillRef.current;
      if (!el) return;
      if (!el.matches(':hover')) setHoveredDelayed(false);
    };
    const id = window.setInterval(check, 180);
    const onBlur = () => setHoveredDelayed(false);
    window.addEventListener('blur', onBlur);
    document.addEventListener('visibilitychange', onBlur);
    return () => {
      window.clearInterval(id);
      window.removeEventListener('blur', onBlur);
      document.removeEventListener('visibilitychange', onBlur);
    };
  }, [hovered]);

  // We only force the bar open during phases the user needs feedback for — active
  // capture/inference. Success, error, and cancelled briefly flash the state pill,
  // then the bar collapses. Keeping the bar expanded until Idle meant a dropped
  // event or stuck Error phase would leave it open forever.
  const activePhase =
    phase.kind === 'listening' ||
    phase.kind === 'processing' ||
    phase.kind === 'warming';
  const terminal =
    phase.kind === 'success' ||
    phase.kind === 'error' ||
    phase.kind === 'cancelled';

  const [terminalUntil, setTerminalUntil] = useState(0);
  useEffect(() => {
    if (terminal) setTerminalUntil(Date.now() + 1400);
  }, [terminal, phase.kind]);
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    if (terminalUntil > Date.now()) {
      const t = window.setTimeout(() => setNow(Date.now()), terminalUntil - Date.now() + 30);
      return () => window.clearTimeout(t);
    }
  }, [terminalUntil]);

  // `dragging` forces the bar open so the user can see what they're dragging.
  // Previously, starting a drag moved the cursor off the pill, which fired
  // mouseleave → hover=false → collapse — the bar would snap shut mid-drag
  // and pop back open on drop. Now it holds its expanded state for the full
  // drag cycle and animates smoothly back closed once the hover grace period
  // expires.
  const expanded = hovered || dragging || activePhase || terminalUntil > now;

  // Mark this window's document so globals.css can strip the dark body background,
  // letting the transparent window show through everywhere except the pill itself.
  useLayoutEffect(() => {
    document.documentElement.classList.add('floating-shell');
    return () => {
      document.documentElement.classList.remove('floating-shell');
    };
  }, []);

  // Click-passthrough poll. The floating window is 520 × 64 px but the pill
  // inside it is only ~210 px collapsed / ~500 px expanded, so the transparent
  // margins either side were a dead click zone over the desktop — users reported
  // icons + background-window clicks that sat under those margins simply not
  // registering. Fix: toggle `setIgnoreCursorEvents` based on whether the cursor
  // is physically over the pill's on-screen bounding rect.
  //
  // The poll runs at ~30 Hz; the per-tick cost is a single cheap IPC invoke
  // (`get_cursor_pos` → GetCursorPos on Windows) plus a rect-containment check.
  // When dragging, we force passthrough off no matter what — lose this and
  // `startDragging()` refuses the hand-off mid-gesture.
  //
  // Dragging + activePhase are read via refs so a phase change mid-session
  // doesn't tear the poll down and re-register it (which would flash
  // passthrough off briefly and cause a visible click flicker).
  const draggingRef = useRef(dragging);
  const activePhaseRef = useRef(activePhase);
  useEffect(() => {
    draggingRef.current = dragging;
  }, [dragging]);
  useEffect(() => {
    activePhaseRef.current = activePhase;
  }, [activePhase]);
  useEffect(() => {
    let cancelled = false;
    let lastPassthrough: boolean | null = null;
    let timer: number | undefined;

    const tick = async () => {
      if (cancelled) return;
      try {
        const el = pillRef.current;
        const win = getCurrentWindow();
        if (!el) {
          timer = window.setTimeout(tick, 120);
          return;
        }
        // Pill's screen-space rect = window origin + DOM-relative rect.
        // outer_position and GetCursorPos are both in physical px so they compose.
        const [pos, scale, cursor] = await Promise.all([
          win.outerPosition(),
          win.scaleFactor(),
          invoke<[number, number]>('get_cursor_pos'),
        ]);
        const r = el.getBoundingClientRect();
        const left = pos.x + Math.round(r.left * scale);
        const top = pos.y + Math.round(r.top * scale);
        const right = pos.x + Math.round(r.right * scale);
        const bottom = pos.y + Math.round(r.bottom * scale);
        // Small padding so a pixel-perfect edge click still lands on the pill.
        const PAD = 2;
        const [cx, cy] = cursor;
        const over =
          cx >= left - PAD && cx <= right + PAD && cy >= top - PAD && cy <= bottom + PAD;
        // Dragging always forces events through so `startDragging()` and the
        // framer drag-lift logic keep their input stream. Active capture
        // phases also keep the bar live so the Stop / Cancel tiles are
        // always one click away.
        const wantPassthrough = !(over || draggingRef.current || activePhaseRef.current);
        if (wantPassthrough !== lastPassthrough) {
          lastPassthrough = wantPassthrough;
          await invoke('set_floating_passthrough', { passthrough: wantPassthrough }).catch(
            () => undefined,
          );
        }
      } catch {
        // Any error in the poll shouldn't brick the bar — worst case the
        // passthrough stays at its last value for a tick.
      }
      if (!cancelled) timer = window.setTimeout(tick, 33);
    };
    // Start with passthrough ON so clicks route through the transparent margins
    // from the moment the window mounts; the poll will flip it off as soon as
    // the cursor enters the pill.
    void invoke('set_floating_passthrough', { passthrough: true }).catch(() => undefined);
    lastPassthrough = true;
    timer = window.setTimeout(tick, 33);

    return () => {
      cancelled = true;
      if (timer !== undefined) window.clearTimeout(timer);
      // Leave passthrough OFF on unmount — we never want to hand over a
      // click-through window to a different React tree without its consent.
      void invoke('set_floating_passthrough', { passthrough: false }).catch(() => undefined);
    };
  }, []);

  // Mount-time clamp. Windows can restore the floating window at a saved
  // position that's no longer on-screen — classic triggers:
  //   • monitor unplugged since last session
  //   • DPI or resolution changed
  //   • the bar was dragged behind the taskbar immediately before quit
  //   • the window config's width grew between versions, clipping the right
  //     edge off a previously-visible position
  // Without this, the user relaunches the app, doesn't see the bar, and has
  // no taskbar affordance to recover it (we set `skipTaskbar: true`). The
  // tray-menu "Reset Utterbar position" escape hatch catches this, but most
  // users won't think to look there. Clamping on mount makes the vast
  // majority of "where did my bar go?" events self-heal.
  useEffect(() => {
    void invoke('clamp_floating_position').catch(() => undefined);
  }, []);

  useEffect(() => {
    if (phase.kind !== 'listening') return;
    const id = window.setInterval(() => tick(), 100);
    return () => window.clearInterval(id);
  }, [phase.kind, tick]);

  // Escape as a no-configuration universal cancel. The global shortcut for
  // cancel (Ctrl+Alt+X by default) works from anywhere, but when the user is
  // already looking at the floating bar the fastest reflex is to slap Escape.
  // We scope it to the bar's own document so it only fires when the bar has
  // focus or the user hovers — nothing else on the system sees the keypress.
  // Only active while recording/processing; no-op in idle and terminal states
  // so a stray Escape after the fact can't trigger another ghost cancel.
  useEffect(() => {
    if (phase.kind !== 'listening' && phase.kind !== 'processing') return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return;
      e.preventDefault();
      void invoke('cancel_recording').catch(() => undefined);
    };
    window.addEventListener('keydown', onKey, { capture: true });
    return () => window.removeEventListener('keydown', onKey, { capture: true });
  }, [phase.kind]);

  const onToggle = async () => {
    if (phase.kind === 'idle' || phase.kind === 'success' || phase.kind === 'cancelled') {
      await invoke('start_recording', { deviceId: null });
    } else if (phase.kind === 'listening') {
      await invoke('stop_recording');
    }
  };
  const onCancel = () => void invoke('cancel_recording');
  const onSettings = () => void invoke('show_main_at', { path: '/settings/general' });

  // "Copy last" — puts the most recently dictated text back on the clipboard
  // so the user can paste it again anywhere. Replaces the old History button:
  // the fast-path someone actually wants from the bar isn't "open the app on
  // a table of past dictations", it's "put the thing I just said back on my
  // clipboard". Flashes a check glyph for 1.2 s on success.
  const [copied, setCopied] = useState(false);
  const copyFlashRef = useRef<number | undefined>(undefined);
  useEffect(() => {
    return () => {
      if (copyFlashRef.current !== undefined) window.clearTimeout(copyFlashRef.current);
    };
  }, []);
  const onCopyLast = async () => {
    if (!lastText) return;
    try {
      await navigator.clipboard.writeText(lastText);
      setCopied(true);
      if (copyFlashRef.current !== undefined) window.clearTimeout(copyFlashRef.current);
      copyFlashRef.current = window.setTimeout(() => setCopied(false), 1200);
    } catch {
      // Clipboard write can fail if the webview has no focus permission;
      // silent is fine — nothing to recover here.
    }
  };

  // Size preference from settings: compact shrinks padding + internal gaps, standard is
  // the default roomier layout.
  const barSize = useSettings((x) => x.data?.general?.bar_size) ?? 'standard';
  const isCompact = barSize === 'compact';
  const barOutline = useSettings((x) => x.data?.general?.bar_outline) ?? true;
  const barVisibility = useSettings((x) => x.data?.general?.bar_visibility) ?? 'always';

  // Drag handler: replaces the CSS `-webkit-app-region: drag` approach. The CSS property
  // comes with two unwanted behaviours on Windows — double-click maximises the window
  // (which snapped the bar to top-centre), and right-click opens the system menu (whose
  // Minimize option stranded the bar off-screen). Calling `startDragging()` on mousedown
  // gives us clean drag semantics with none of that baggage. Buttons still opt out via
  // the `no-drag` class — we bail early if the event target sits inside one.
  const onDragPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement | null;
    if (target && target.closest('.no-drag')) return;
    // Stop any latent double-click-to-maximize side effects before they fire.
    if (e.detail >= 2) {
      e.preventDefault();
      return;
    }
    e.preventDefault();
    setDragging(true);
    void getCurrentWindow().startDragging().catch(() => undefined);
  };

  // Clamp the bar back into the current monitor's work-area after any drag. Tauri v2
  // emits `onMoved` continuously while dragging, so we debounce — only invoke the
  // clamp command once the user has stopped moving. This is the guard against the
  // "bar stuck behind the taskbar" bug: no matter where they let go, we pull it back
  // into a visible rectangle within 200 ms.
  useEffect(() => {
    let timer: number | undefined;
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void getCurrentWindow()
      .onMoved(() => {
        if (timer !== undefined) window.clearTimeout(timer);
        timer = window.setTimeout(() => {
          void invoke('clamp_floating_position').catch(() => undefined);
        }, 200);
      })
      .then((u) => {
        if (cancelled) u();
        else unlisten = u;
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
      if (timer !== undefined) window.clearTimeout(timer);
      unlisten?.();
    };
  }, []);

  // Apply bar visibility preference. We drive show/hide from the frontend because the
  // window is already alive (React state, listeners) — we just need the OS window to be
  // mapped on screen or not.
  //
  // The `unminimize()` before `show()` is non-obvious but load-bearing: Windows' Win+D,
  // Win+M, and certain shell messages can send WM_SYSCOMMAND SC_MINIMIZE to every top-
  // level window including ours — `minimizable: false` in the Tauri config only removes
  // the UI caption affordance, it does not block the system command. Once minimised, a
  // plain `show()` does NOT restore the window; it remains minimised and therefore
  // invisible (we set `skipTaskbar: true`, so there's no taskbar entry to click on
  // either — the pill is effectively gone). Unminimising first recovers it reliably.
  useEffect(() => {
    const win = getCurrentWindow();
    const shouldShow = (() => {
      if (barVisibility === 'hidden') return false;
      if (barVisibility === 'while_recording') {
        return activePhase || terminalUntil > now;
      }
      return true;
    })();
    if (shouldShow) {
      void (async () => {
        try {
          await win.unminimize();
        } catch {
          // not minimised → harmless
        }
        try {
          await win.show();
          await win.setAlwaysOnTop(true);
          // Re-clamp after every show. The bar can be hidden/shown many times
          // in a session under `while_recording`, and between hide and show
          // the OS may have moved the window (monitor change, DPI event) —
          // we don't want it to re-appear off-screen the next time recording
          // starts. Invoking clamp after show() catches exactly that.
          await invoke('clamp_floating_position').catch(() => undefined);
        } catch {
          // nothing to do
        }
      })();
    } else {
      void win.hide().catch(() => undefined);
    }
  }, [barVisibility, activePhase, terminalUntil, now]);

  return (
    <div
      className="flex h-screen w-screen items-start justify-center bg-transparent p-0"
      // Block the webview's default right-click menu on the floating window. Without
      // this the user gets "Save image", "Copy image", "Inspect" — and Save image
      // rasterises the transparent surface as a speckled blue-dot pattern (the
      // browser's transparent-pixel checkerboard bleeds through). There's nothing
      // useful in the menu for a pill-shaped single-purpose widget.
      onContextMenu={(e) => e.preventDefault()}
    >
      <motion.div
        ref={pillRef}
        layout
        onMouseEnter={() => setHoveredDelayed(true)}
        onMouseLeave={() => setHoveredDelayed(false)}
        onPointerLeave={() => setHoveredDelayed(false)}
        onPointerDown={onDragPointerDown}
        // `onPointerUp` NOT wired here — Tauri's startDragging()
        // synthesises a pointerup on the webview the instant the OS
        // takes over the window move, which would clear the drag state
        // mid-drag. Termination is handled by the debounced onMoved
        // listener in the drag effect above.
        initial={{ scale: 0.96, opacity: 0, y: -12 }}
        animate={{
          // Drag-lift: pill scales up noticeably while dragging (1.05 —
          // more visible than the v1 1.03) and settles back with a small
          // spring overshoot on release. Combined with the accent-border
          // + accent-glow-halo swap below, this is the "picks up off the
          // desktop" moment.
          scale: dragging ? 1.05 : 1,
          opacity: 1,
          y: 0,
        }}
        transition={{
          // Mount + drag-lift share the scale track. The spring gives the
          // drag release a satisfying little overshoot back to rest.
          scale:   { type: 'spring', stiffness: 460, damping: 28, mass: 0.7 },
          opacity: { duration: 0.28, ease: [0.22, 1, 0.36, 1] },
          y:       { duration: 0.28, ease: [0.22, 1, 0.36, 1] },
          // Layout owns the expand/collapse width transition. A predictable
          // duration-based tween is used instead of a spring here because
          // the inner-content AnimatePresence runs on its own timeline —
          // a spring "settle" at the end of the layout animation meant
          // the icons had already finished fading out while the pill
          // was still wobbling to a stop, which reads as a staggered,
          // three-stage close. A 280 ms tween finishes at a known instant
          // so the inner mode="wait" exit (180 ms) completes just inside
          // the window, and the eye sees one smooth close rather than
          // sequential stages.
          layout:  { duration: 0.28, ease: [0.22, 1, 0.36, 1] },
        }}
        style={{
          // Tell the compositor what's going to change so transitions
          // stay GPU-accelerated. box-shadow is included so the drag
          // ring + busy-flash appear instantly even while Tauri's
          // startDragging() has stalled the main-DOM paint loop.
          willChange: 'transform, width, box-shadow',
          // The prototype's signature chrome: a top→bottom gradient driven by
          // the theme tokens `--pill-top` / `--pill-bot`, so every theme
          // (Graphite, Obsidian, Parchment…) paints its own pill. Replaces
          // the old blurred-glass stack.
          background: 'linear-gradient(180deg, var(--pill-top), var(--pill-bot))',
          // Border:
          //   • Default: `--pill-stroke` — on dark-pill themes this is
          //     white @ 72% (matching the pill-mark's rim in the logo so
          //     the floating bar reads as "the logo, full-size"); on
          //     Paper's light pill it flips to dark ink @ 28% so the
          //     outline stays visible without punching through.
          //   • Outline-off: a quieter hairline (`--pill-stroke-soft`) so
          //     the user's "I don't want a ring" preference is still
          //     honoured on either pill tone.
          // The dragging-accent border is NOT applied via this inline
          // `borderColor` swap — Tauri's `startDragging()` hands the window
          // move to the OS immediately, and while the OS is moving the
          // window the webview's paint pipeline stalls, so a freshly
          // re-computed style on the border is not visible until drag
          // release. Instead we overlay an absolutely-positioned accent
          // ring (see below inside the motion.div) whose `opacity` toggles
          // on `dragging`; that ring sits on its own compositor layer and
          // a GPU-side opacity change is honoured by the shell even while
          // the main DOM is paused mid-drag.
          borderColor: barOutline
            ? 'var(--pill-stroke)'
            : 'var(--pill-stroke-soft)',
          // Shadow:
          //   • Rest: a tight hairline shadow (2px / 8px blur). The old
          //     stack of 14/32 + 2/6 was producing a big dark halo that
          //     read as a boxy rectangle sitting behind the pill on bright
          //     desktops — the transparent-window + large soft shadow
          //     combo doesn't render cleanly there. Small and close wins.
          //   • Expanded: swap the hairline for an inner accent rim —
          //     that's where the "I'm live" glow lives — plus a still-
          //     modest outer shadow. No stacked offsets.
          //   • Dragging: a solid-accent outer ring painted flush against
          //     the existing 2px border (box-shadow with spread=2 and no
          //     blur) so the whole pill's perimeter reads as cyan, plus
          //     an outer accent halo. Box-shadow is on the GPU compositor
          //     so the ring becomes visible instantly even while Tauri's
          //     startDragging() has stalled the webview's main-DOM paint.
          //     The inner-rim glow comes through underneath for depth.
          // Box-shadow composition:
          //   dragging / busyFlash: solid accent ring flush with the 2px
          //     border + soft accent halo + inner cyan rim-glow + ground
          //     shadow. Makes the whole pill perimeter read cyan.
          //   expanded (no drag / flash): inner accent rim-glow only —
          //     subtle "charged" state.
          //   collapsed idle: tight hairline shadow.
          //
          // `busyFlash` piggybacks on the drag ring so the user gets
          // the SAME visual cue whether they held the pill or tapped a
          // hotkey while busy. 900ms of flash, cleared by setBusyFlash.
          //
          // `transition` lives INSIDE the style object but note that
          // framer-motion does apply inline-style transitions when
          // there's no conflicting motion track — box-shadow isn't in
          // `animate={}` so the browser handles the transition
          // directly, GPU-composited.
          boxShadow: (dragging || busyFlash)
            ? [
                '0 0 0 2px var(--accent)',
                '0 0 0 6px color-mix(in oklab, var(--accent) 22%, transparent)',
                'inset 0 0 22px color-mix(in oklab, var(--accent) 32%, transparent)',
                '0 6px 18px rgba(0,0,0,0.28)',
              ].join(', ')
            : expanded
              ? 'inset 0 0 20px color-mix(in oklab, var(--accent) 14%, transparent), 0 3px 10px rgba(0,0,0,0.26)'
              : '0 2px 8px rgba(0,0,0,0.24)',
          transition: 'box-shadow 180ms ease-out',
        }}
        className={cn(
          // Height scales with size preference. Standard is 52px (prototype
          // default). Compact drops to 44px — a real ~15% size step that the
          // user feels as "noticeably smaller", not just a padding tweak.
          //
          // Layout strategy:
          //   - Collapsed: pill sizes to fit logo + wordmark, no min-w.
          //     `gap-2.5` spaces logo and wordmark.
          //   - Expanded: `justify-between` pushes the left cluster
          //     (logo+wordmark) to the left edge and the right cluster
          //     (actions) to the right edge, with a flexible drag-handle
          //     region in the middle. The user can grab that middle area
          //     to move the window without accidentally hitting a button.
          //     min-w keeps the pill wide enough that the drag region
          //     never collapses to zero even with a minimal action
          //     cluster (idle state: StatePill + 3 buttons).
          'relative mt-0.5 flex items-center overflow-hidden rounded-full border-2',
          isCompact ? 'h-[44px]' : 'h-[52px]',
          expanded
            ? isCompact
              ? 'pl-1.5 pr-1 gap-2 min-w-[400px] justify-between'
              : 'pl-3 pr-2 gap-2.5 min-w-[540px] justify-between'
            : isCompact
              ? 'px-2 gap-1.5'
              : 'px-3 gap-2.5',
        )}
      >
        {/* LEFT — always visible. Waveform-only glyph (the pill body + hole-
            punch from the full logo are deliberately dropped here: the
            floating bar already IS a pill, and rendering a miniature pill
            inside it reads as an app-tile-within-a-bar — DESIGN_SYSTEM §11.8
            "Don't repeat-brand inside the app"). Plus the "Utter·bar"
            wordmark where the "bar" half takes the accent gradient so the
            logo re-colours with the theme. Kept tight so collapsed width
            stays ~210px. */}
        <div className="flex shrink-0 items-center gap-2">
          <UtterbarMiniMark
            active={phase.kind === 'listening'}
            size={isCompact ? 18 : 24}
            // Soft drop-shadow + accent halo so the bars lift off the pill
            // gradient — same feel the old inline mark had, minus the pill
            // body that was causing the "mini pill inside a pill" read.
            className="[filter:drop-shadow(0_1px_2px_rgba(0,0,0,0.35))_drop-shadow(0_0_8px_color-mix(in_oklab,var(--accent)_28%,transparent))] ml-1"
          />
          <span
            className={cn(
              'cursor-default select-none font-semibold tracking-[-0.01em] whitespace-nowrap',
              isCompact ? 'text-[15px]' : 'text-[19px]',
            )}
            style={{ color: 'var(--pill-fg)' }}
          >
            Utter
            <span
              // The "bar" half is painted with the accent→accent-2 gradient
              // via background-clip. Safari/Chromium both support this on a
              // `span` as long as display is inline-block or the element
              // has clip-path. `bg-clip-text` + `text-transparent` is the
              // canonical Tailwind recipe.
              className="bg-clip-text text-transparent"
              style={{
                backgroundImage:
                  'linear-gradient(180deg, var(--accent-2), var(--accent))',
              }}
            >
              bar
            </span>
          </span>
        </div>

        {/* RIGHT — expanded content. AnimatePresence gates mount; the outer
            motion.div's `layout` tween handles width in a fixed 280ms
            window. The inner tween runs SYNCHRONOUSLY with the layout
            rather than nesting inside it: a pure opacity fade with no
            delay / scale / translate, same 280ms duration, same easing.
            That way the pill width and the content opacity read as a
            SINGLE motion — previously the content faded in a separate
            short burst (180ms exit, 200ms enter with 80ms delay) while
            the pill resized on its own timeline, which the eye still
            picked up as two sequential stages no matter how short.
            Matching the timelines is the only way to produce one
            cohesive motion. */}
        <AnimatePresence initial={false}>
          {expanded && (
            <motion.div
              key={`expanded-${phase.kind === 'listening' ? 'rec' : 'idle'}`}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }}
              // `min-w-0` + `overflow-hidden` let the content clip cleanly
              // as the pill closes. No `flex-1` or `justify-end` — those
              // were the source of the empty gap between "Utterbar" and
              // the recording cluster on the expanded state: flex-1 pushed
              // the action cluster to the right edge, justify-end kept it
              // there, and the empties between read as wasted real estate.
              className="flex min-w-0 items-center gap-2 overflow-hidden whitespace-nowrap"
            >
              {phase.kind === 'listening' ? (
                <>
                  {/* Red ripple dot + mono timer + seven-bar speech wave.
                      The ripple, timer and wave together form the prototype's
                      "we are recording RIGHT NOW" cluster — three separate
                      cues (colour, number, motion) so the state reads at any
                      glance angle. */}
                  <span className="ub-bar-ripple shrink-0" aria-hidden />
                  <span
                    className="cursor-default select-none font-mono text-[13px] tabular-nums -tracking-[0.01em]"
                    style={{ color: 'var(--pill-fg-muted)' }}
                  >
                    {formatDuration(elapsed)}
                  </span>
                  <div className="ub-bar-wave ml-1 mr-1 shrink-0" aria-hidden>
                    <span className="ub-bar-wave__bar" />
                    <span className="ub-bar-wave__bar" />
                    <span className="ub-bar-wave__bar" />
                    <span className="ub-bar-wave__bar" />
                    <span className="ub-bar-wave__bar" />
                    <span className="ub-bar-wave__bar" />
                    <span className="ub-bar-wave__bar" />
                  </div>
                  <div className="flex items-center gap-1">
                    <BarButton
                      onClick={onToggle}
                      label="Stop"
                      icon={CircleStop}
                      tone="rose"
                      compact={isCompact}
                    />
                    <BarButton onClick={onCancel} label="Cancel" icon={X} compact={isCompact} />
                    <Divider />
                    <BarButton
                      onClick={onCopyLast}
                      label={copied ? 'Copied' : 'Copy last message'}
                      icon={copied ? Check : CopyIcon}
                      compact={isCompact}
                      tone={copied ? 'accent' : 'default'}
                      disabled={!lastText}
                    />
                    <BarButton onClick={onSettings} label="Settings" icon={SettingsIcon} compact={isCompact} />
                  </div>
                </>
              ) : (
                <>
                  {/* Non-listening expanded state: StatePill shows the phase
                      (Ready / Warming / Processing / Inserted / Error /
                      Cancelled) and the action row offers Record + History +
                      Settings. */}
                  <div className="flex flex-1 items-center justify-start">
                    <StatePill phase={phase} />
                  </div>
                  <div className="flex items-center gap-1">
                    <BarButton
                      onClick={onToggle}
                      label="Record"
                      icon={SquarePlay}
                      tone="accent"
                      compact={isCompact}
                    />
                    <Divider />
                    <BarButton
                      onClick={onCopyLast}
                      label={copied ? 'Copied' : 'Copy last message'}
                      icon={copied ? Check : CopyIcon}
                      compact={isCompact}
                      tone={copied ? 'accent' : 'default'}
                      disabled={!lastText}
                    />
                    <BarButton onClick={onSettings} label="Settings" icon={SettingsIcon} compact={isCompact} />
                  </div>
                </>
              )}
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>
    </div>
  );
}

/**
 * Bar action button — prototype v2.
 *
 *   • 34×34, rounded-[10px] (squircle-ish rather than pill — v1 used
 *     rounded-full which read as "another tiny pill inside the big pill").
 *   • Hover: scale 1.08 with a subtle surface lift. The overshoot-curve
 *     cubic-bezier makes the hover pop without wobbling.
 *   • Active: scale 0.86 — the "squish" that makes clicks feel mechanical.
 *     Combined with `transition-transform` this happens entirely in CSS on
 *     the main thread, no framer state needed.
 *   • Tone: `default` is a neutral glass tile, `accent` / `rose` are lit
 *     tiles that burn the call-to-action quota. Only one lit tile is ever
 *     visible at a time (Record XOR Stop), which keeps the "cyan budget"
 *     from blowing out.
 */
function BarButton({
  onClick,
  label,
  icon: Icon,
  tone = 'default',
  compact = false,
  disabled = false,
}: {
  onClick: () => void;
  label: string;
  icon: LucideIcon;
  tone?: 'default' | 'accent' | 'rose';
  compact?: boolean;
  disabled?: boolean;
}) {
  const toneStyle: React.CSSProperties =
    tone === 'accent'
      ? {
          background: 'linear-gradient(180deg, var(--accent-2), var(--accent))',
          color: '#06121a',
          boxShadow: '0 2px 10px color-mix(in oklab, var(--accent) 40%, transparent)',
        }
      : tone === 'rose'
        ? {
            background: 'linear-gradient(180deg, color-mix(in oklab, var(--rose) 85%, #fff), var(--rose))',
            color: '#1a0608',
            boxShadow: '0 2px 10px color-mix(in oklab, var(--rose) 40%, transparent)',
          }
        : {
            // Neutral buttons use the pill-local surface/ink family so
            // they invert cleanly on Paper's light pill (dark ink on a
            // barely-there warm-grey tile) instead of staying white-on-
            // white-on-paper.
            background: 'var(--pill-surface)',
            color: 'var(--pill-fg-muted)',
          };
  return (
    <button
      onClick={onClick}
      aria-label={label}
      title={label}
      disabled={disabled}
      style={toneStyle}
      className={cn(
        // no-drag opts out of the drag handler so clicking a button never
        // starts a window move.
        'no-drag group/btn flex items-center justify-center rounded-[10px]',
        // Compact bar uses 28×28 tiles with a tighter corner radius so the
        // action cluster doesn't crowd out the mini-mark + wordmark inside
        // the shorter (44px) pill. Standard stays at the prototype's 34×34.
        compact ? 'h-[28px] w-[28px] rounded-[8px]' : 'h-[34px] w-[34px]',
        // Spring-ish hover/active. The overshoot cubic-bezier
        // (0.34, 1.56, 0.64, 1) gives the hover a tiny "pop" and the active
        // shrink a satisfying bottom-out without going full Framer.
        'transition-[transform,box-shadow,background,color] duration-150',
        '[transition-timing-function:cubic-bezier(0.34,1.56,0.64,1)]',
        !disabled && 'hover:scale-[1.08] active:scale-[0.86]',
        !disabled &&
          tone === 'default' && 'hover:bg-[var(--pill-surface-hi)] hover:text-[var(--pill-ink-on-hi)]',
        !disabled && (tone === 'accent' || tone === 'rose') && 'hover:brightness-110',
        // Disabled = no previous dictation yet. Dim the tile, block hover
        // effects, and show the not-allowed cursor so the user knows why
        // the click is doing nothing.
        disabled && 'cursor-not-allowed opacity-40',
      )}
    >
      <Icon
        className={compact ? 'h-[15px] w-[15px]' : 'h-[18px] w-[18px]'}
        strokeWidth={2.25}
      />
    </button>
  );
}

/**
 * 1px inset divider between primary actions (Stop/Cancel/Record) and the
 * "window" actions (History/Settings). Keeps the two clusters legibly
 * grouped without adding visual noise.
 */
function Divider() {
  return (
    <span
      aria-hidden
      className="mx-0.5 block h-5 w-px"
      style={{ background: 'var(--pill-dim)' }}
    />
  );
}

