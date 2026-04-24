/**
 * Cursor + ClickPulse — shared demo primitives so every feature comp
 * uses the same cursor shape and click affordance.
 *
 * The cursor is a Windows-style arrow (not macOS — Utterbar is
 * Windows-first). Rendered inline as SVG so it survives MP4 export
 * without needing a PNG asset. A soft drop-shadow keeps it legible
 * on both bg-0 and bg-1 surfaces.
 *
 * ClickPulse is an expanding cyan ring that fades over ~400 ms —
 * bright and unmissable at 400 px mobile render size. The brief
 * calls for "obvious" interaction feedback; a subtle ripple would
 * defeat the point.
 *
 * Coordinates are expressed in composition space (1920×1080 for
 * videos). Callers drive `x`/`y` with `tween` so paths interpolate
 * the same way every other motion does.
 */
import React from 'react';
import { tween } from './anim';

export function Cursor({
  x,
  y,
  size = 56,
}: {
  x: number;
  y: number;
  /** visual size — larger = friendlier on mobile renders. Default 56 px. */
  size?: number;
}) {
  // Preserve aspect ratio of the Windows arrow glyph (12×16 viewBox).
  const w = size * 0.75;
  const h = size;
  return (
    <svg
      width={w}
      height={h}
      viewBox="0 0 12 16"
      style={{
        position: 'absolute',
        left: x,
        top: y,
        filter: 'drop-shadow(0 3px 10px rgba(0,0,0,0.8))',
        pointerEvents: 'none',
      }}
    >
      {/* Classic Windows arrow: hotspot at the tip (0,0 in viewBox). */}
      <path
        d="M 1.3 0.8 L 1.3 12.8 L 4.4 9.9 L 6.6 14.6 L 8.6 13.6 L 6.4 9 L 10.5 9 Z"
        fill="#FFFFFF"
        stroke="#000000"
        strokeWidth="0.5"
        strokeLinejoin="round"
      />
    </svg>
  );
}

/**
 * ClickPulse — an expanding cyan ring centred on (x,y). `startFrame`
 * is when the click happens; the ring grows + fades over `durationFrames`.
 * Renders nothing outside its active window so callers can mount it
 * unconditionally.
 */
export function ClickPulse({
  frame,
  startFrame,
  x,
  y,
  durationFrames = 12,
  startRadius = 40,
  endRadius = 140,
  colour = 'rgba(34, 211, 238,',
}: {
  frame: number;
  startFrame: number;
  x: number;
  y: number;
  durationFrames?: number;
  startRadius?: number;
  endRadius?: number;
  /** rgba prefix without the alpha — caller's chance to pick a different tint. */
  colour?: string;
}) {
  const endFrame = startFrame + durationFrames;
  if (frame < startFrame || frame >= endFrame) return null;

  const r = tween(frame, startFrame, endFrame, startRadius, endRadius);
  const opacity = tween(frame, startFrame, endFrame, 1, 0);

  return (
    <div
      style={{
        position: 'absolute',
        left: x - r,
        top: y - r,
        width: r * 2,
        height: r * 2,
        borderRadius: 999,
        border: `4px solid ${colour} ${opacity})`,
        pointerEvents: 'none',
      }}
    />
  );
}

/**
 * cursorPath — helper to animate the cursor through a list of
 * waypoints with hold times between them. Returns the (x, y) for the
 * current frame. Points are hit in order; `holdFrames` is how long
 * the cursor lingers after reaching each point before the next
 * transit starts.
 *
 * Example:
 *   cursorPath(frame, [
 *     { x: 1800, y: 1000, t: 0 },           // off-frame at t=0
 *     { x: 960,  y: 500,  t: f(1.0) },      // arrive at input by 1.0s
 *     { x: 1580, y: 500,  t: f(2.5) },      // slide to Add by 2.5s
 *   ]);
 */
export function cursorPath(
  frame: number,
  waypoints: { x: number; y: number; t: number }[],
): { x: number; y: number } {
  if (waypoints.length === 0) return { x: 0, y: 0 };
  if (frame <= waypoints[0].t) return { x: waypoints[0].x, y: waypoints[0].y };
  for (let i = 0; i < waypoints.length - 1; i++) {
    const a = waypoints[i];
    const b = waypoints[i + 1];
    if (frame >= a.t && frame <= b.t) {
      return {
        x: tween(frame, a.t, b.t, a.x, b.x),
        y: tween(frame, a.t, b.t, a.y, b.y),
      };
    }
  }
  const last = waypoints[waypoints.length - 1];
  return { x: last.x, y: last.y };
}
