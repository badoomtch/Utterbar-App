import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { useSettings } from '@/state/settings';
import { cn } from '@/lib/cn';

/**
 * Accessibility-focused settings. We split these out of General so users who
 * need them can find them quickly rather than hunting through the
 * appearance cluster, and so we have a clean home for adding more over time
 * (high-contrast theme, captions-always, stronger focus rings, etc.).
 */

const SCALE_STOPS = [90, 100, 115, 125, 150, 175] as const;

export function AccessibilitySettings() {
  const { t } = useTranslation('settings');
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);

  // Hook order must be stable across renders, every hook in this component
  // runs unconditionally on the first line and any early return comes AFTER.
  // Previously the `if (!s) return null` guard sat above the useState/useEffect
  // below, which meant the very first render (before settings loaded) took a
  // different hook path than subsequent renders and the react-hooks lint
  // rule (correctly) flagged it.
  const g = (s?.general ?? {}) as Record<string, unknown>;
  const committedScale = clampScale(Number(g.ui_scale ?? 100));
  // Local draft state for the slider. Committing on every `onChange` while the
  // user drags meant the entire app re-flowed 20+ times per second (ui_scale
  // writes to `document.documentElement.style.fontSize`, which invalidates
  // layout). That read as the whole window jittering side-to-side, the
  // slider itself would re-lay-out under the cursor as padding around it grew
  // and shrank. We now keep the draft value in component state and only
  // commit when the user releases the thumb (onPointerUp / onKeyUp).
  const [draftScale, setDraftScale] = useState(committedScale);
  useEffect(() => {
    setDraftScale(committedScale);
  }, [committedScale]);

  if (!s) return null;

  const uiScale = draftScale;

  const update = (key: string, value: unknown) =>
    patch({ general: { ...s.general, [key]: value } });

  const commitScale = (next: number) => {
    const clamped = clampScale(next);
    setDraftScale(clamped);
    if (clamped !== committedScale) void update('ui_scale', clamped);
  };

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader
        title={t('accessibility.title')}
        subtitle={t('accessibility.subtitle')}
      />

      <div className="mt-6 space-y-4">
        <Card
          title={t('accessibility.ui_scale.title')}
          description={t('accessibility.ui_scale.description')}
        >
          <Row
            label={t('accessibility.ui_scale.scale.label')}
            description={t('accessibility.ui_scale.scale.description')}
          >
            <div className="flex flex-wrap items-center gap-2">
              {SCALE_STOPS.map((pct) => (
                <button
                  key={pct}
                  onClick={() => commitScale(pct)}
                  className={cn('ub-chip font-mono', committedScale === pct && 'ub-chip--active')}
                >
                  {t('accessibility.ui_scale.value', { value: pct })}
                </button>
              ))}
            </div>
          </Row>
          <Row label={t('accessibility.ui_scale.fine_tune.label')}>
            <div className="flex w-full items-center gap-3">
              <input
                type="range"
                min={90}
                max={175}
                step={5}
                value={uiScale}
                // Update local preview only while dragging, the visible
                // number in the readout still tracks the thumb, but we don't
                // touch the document font-size (and therefore don't re-flow
                // the whole window) until the user lets go. This is what
                // kills the "UI shakes as I drag" behaviour.
                onChange={(e) => setDraftScale(clampScale(Number(e.target.value)))}
                onPointerUp={(e) => commitScale(Number((e.target as HTMLInputElement).value))}
                onKeyUp={(e) => commitScale(Number((e.target as HTMLInputElement).value))}
                onBlur={(e) => commitScale(Number(e.target.value))}
                className="flex-1"
              />
              <span className="w-14 text-right font-mono text-[12px] tabular-nums text-fg-hi">
                {t('accessibility.ui_scale.value', { value: uiScale })}
              </span>
            </div>
          </Row>
        </Card>

        <Card
          title={t('accessibility.motion.title')}
          description={t('accessibility.motion.description')}
        >
          <Row
            label={t('accessibility.motion.reduce_motion.label')}
            description={t('accessibility.motion.reduce_motion.description')}
          >
            <Toggle
              checked={Boolean(g.reduce_motion)}
              onChange={(v) => void update('reduce_motion', v)}
            />
          </Row>
        </Card>

        <Card
          title={t('accessibility.contrast.title')}
          description={t('accessibility.contrast.description')}
        >
          <Row
            label={t('accessibility.contrast.high_contrast.label')}
            description={t('accessibility.contrast.high_contrast.description')}
          >
            <Toggle
              checked={Boolean(g.high_contrast)}
              onChange={(v) => void update('high_contrast', v)}
            />
          </Row>
          <Row
            label={t('accessibility.contrast.larger_focus_rings.label')}
            description={t('accessibility.contrast.larger_focus_rings.description')}
          >
            <Toggle
              checked={Boolean(g.larger_focus_rings)}
              onChange={(v) => void update('larger_focus_rings', v)}
            />
          </Row>
        </Card>

        <Card
          title={t('accessibility.transparency.title')}
          description={t('accessibility.transparency.description')}
        >
          <Row
            label={t('accessibility.transparency.reduce_transparency.label')}
            description={t('accessibility.transparency.reduce_transparency.description')}
          >
            <Toggle
              checked={Boolean(g.reduce_transparency)}
              onChange={(v) => void update('reduce_transparency', v)}
            />
          </Row>
        </Card>
      </div>
    </div>
  );
}

function clampScale(n: number): number {
  if (!Number.isFinite(n)) return 100;
  return Math.min(175, Math.max(90, Math.round(n)));
}
