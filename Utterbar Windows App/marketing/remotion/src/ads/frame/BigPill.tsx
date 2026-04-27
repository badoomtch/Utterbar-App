/**
 * BigPill — wraps BarDriver at a target on-screen scale chosen for
 * vertical mobile, where the pill is a hero element rather than a
 * settings-page accessory. Default scale 2.6× lands the bar at
 * roughly 60% of frame width on a 1080-wide canvas — large enough
 * to read at thumb distance.
 *
 * Centres horizontally; caller positions vertically.
 */
import React from 'react';
import { BarDriver, type BarState } from '../../bar/BarDriver';

type Props = {
  state: BarState;
  elapsedMs?: number;
  scale?: number;
  top?: number;
};

export function BigPill({ state, elapsedMs = 0, scale = 2.6, top }: Props) {
  return (
    <div
      style={{
        position: 'absolute',
        top,
        left: '50%',
        transform: `translateX(-50%) scale(${scale})`,
        transformOrigin: 'top center',
      }}
    >
      <BarDriver state={state} elapsedMs={elapsedMs} />
    </div>
  );
}
