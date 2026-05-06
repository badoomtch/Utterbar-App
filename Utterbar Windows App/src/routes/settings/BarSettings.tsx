import { LocateFixed } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { invoke } from '@/lib/ipc';
import { useSettings } from '@/state/settings';
import { cn } from '@/lib/cn';

const BAR_SIZES = ['compact', 'standard'] as const;

// Stable ids for the visibility options. The id values match the
// `bar_visibility` field's serialised values on the Rust side, so the
// patch wiring continues to write `'always' | 'while_recording' |
// 'hidden'` against `general.bar_visibility`. Visible labels and
// descriptions resolve via t() at render time.
const VISIBILITY_OPTIONS: ReadonlyArray<{
  id: 'always' | 'while_recording' | 'hidden';
}> = [{ id: 'always' }, { id: 'while_recording' }, { id: 'hidden' }];

export function BarSettings() {
  const { t } = useTranslation('settings');
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  if (!s) return null;

  const g = s.general as Record<string, unknown>;
  const update = (key: string, value: unknown) =>
    patch({ general: { ...s.general, [key]: value } });

  const visibility = (g.bar_visibility as string) ?? 'always';
  const size = (g.bar_size as string) ?? 'standard';
  const outline = (g.bar_outline as boolean | undefined) ?? true;

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader
        title={t('bar_settings.title')}
        subtitle={t('bar_settings.subtitle')}
      />
      <div className="mt-6 space-y-4">
        <Card title={t('bar_settings.visibility.title')}>
          {/* Radio tiles — surface elevation + rail for the active one,
              matching the capture-source picker on Audio settings. */}
          <ul className="space-y-2">
            {VISIBILITY_OPTIONS.map((opt) => {
              const selected = visibility === opt.id;
              return (
                <li key={opt.id}>
                  <label
                    className={cn(
                      'group relative flex cursor-pointer items-start gap-3 rounded-input p-4 transition-colors',
                      selected ? 'bg-bg-2' : 'bg-bg-2/40 hover:bg-bg-2/70',
                    )}
                  >
                    <span
                      className={cn(
                        'absolute left-0 top-3 bottom-3 w-[2px] rounded-r-full transition-opacity',
                        selected ? 'bg-accent opacity-100' : 'opacity-0',
                      )}
                      aria-hidden
                    />
                    <input
                      type="radio"
                      checked={selected}
                      onChange={() => void update('bar_visibility', opt.id)}
                      className="mt-1 accent-[var(--accent)]"
                    />
                    <div className="pl-1">
                      <div className="text-[13px] font-bold text-fg-hi">
                        {t(`bar_settings.visibility.${opt.id}.label` as const)}
                      </div>
                      <div className="mt-0.5 text-[12px] leading-relaxed text-muted">
                        {t(`bar_settings.visibility.${opt.id}.description` as const)}
                      </div>
                    </div>
                  </label>
                </li>
              );
            })}
          </ul>
        </Card>

        <Card title={t('bar_settings.appearance.title')}>
          <Row
            label={t('bar_settings.appearance.size.label')}
            description={t('bar_settings.appearance.size.description')}
          >
            <div className="flex gap-2">
              {BAR_SIZES.map((b) => (
                <button
                  key={b}
                  onClick={() => void update('bar_size', b)}
                  className={cn('ub-chip', size === b && 'ub-chip--active')}
                >
                  {t(`bar_settings.appearance.size.options.${b}` as const)}
                </button>
              ))}
            </div>
          </Row>
          <Row
            label={t('bar_settings.appearance.outline.label')}
            description={t('bar_settings.appearance.outline.description')}
          >
            <Toggle checked={outline} onChange={(v) => void update('bar_outline', v)} />
          </Row>
        </Card>

        <Card title={t('bar_settings.position.title')}>
          <Row
            label={t('bar_settings.position.reset.label')}
            description={t('bar_settings.position.reset.description')}
          >
            <button
              onClick={() => void invoke('reset_floating_position').catch(() => undefined)}
              className="ub-btn ub-btn-plain !py-[0.4rem] !text-[12px] inline-flex items-center gap-1.5"
            >
              <LocateFixed className="h-3.5 w-3.5" />
              {t('bar_settings.position.reset.button')}
            </button>
          </Row>
        </Card>
      </div>
    </div>
  );
}
