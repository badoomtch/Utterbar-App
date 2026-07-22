import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { useSettings } from '@/state/settings';

export function TextOutputSettings() {
  const { t } = useTranslation('settings');
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);

  // Renamed from `t` to avoid shadowing the i18next translator above. The
  // structural settings field shape is unchanged — every row still reads
  // `to.<field>` and writes via `update('<field>', v)` against the
  // `text_output.<field>` slot on Settings.
  const to = (s?.text_output ?? {}) as Record<string, boolean | number>;
  const committedClipboardRestoreDelay = Number(to.clipboard_restore_delay_ms ?? 1500);
  const [clipboardRestoreDelay, setClipboardRestoreDelay] = useState(committedClipboardRestoreDelay);

  useEffect(() => {
    setClipboardRestoreDelay(committedClipboardRestoreDelay);
  }, [committedClipboardRestoreDelay]);

  if (!s) return null;

  const update = (key: string, value: unknown) =>
    patch({ text_output: { ...s.text_output, [key]: value } });

  const commitClipboardRestoreDelay = (delay: number) => {
    setClipboardRestoreDelay(delay);
    if (delay !== committedClipboardRestoreDelay) {
      void update('clipboard_restore_delay_ms', delay);
    }
  };

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader title={t('text_output.title')} subtitle={t('text_output.subtitle')} />
      <div className="mt-6 space-y-4">
        <Card title={t('text_output.delivery.title')}>
          <Row
            label={t('text_output.delivery.paste_automatically.label')}
            description={t('text_output.delivery.paste_automatically.description')}
          >
            <Toggle checked={Boolean(to.paste_automatically)} onChange={(v) => void update('paste_automatically', v)} />
          </Row>
          <Row
            label={t('text_output.delivery.restore_clipboard.label')}
            description={t('text_output.delivery.restore_clipboard.description')}
          >
            <Toggle checked={Boolean(to.restore_clipboard)} onChange={(v) => void update('restore_clipboard', v)} />
          </Row>
          <Row
            label={t('text_output.delivery.clipboard_restore_delay.label')}
            description={t('text_output.delivery.clipboard_restore_delay.description', {
              milliseconds: clipboardRestoreDelay,
            })}
          >
            <div className="flex w-full items-center gap-3">
              <input
                type="range"
                min={250}
                max={1500}
                step={250}
                disabled={!to.restore_clipboard}
                value={clipboardRestoreDelay}
                onChange={(e) => setClipboardRestoreDelay(Number(e.target.value))}
                onPointerUp={(e) => commitClipboardRestoreDelay(Number((e.target as HTMLInputElement).value))}
                onKeyUp={(e) => commitClipboardRestoreDelay(Number((e.target as HTMLInputElement).value))}
                onBlur={(e) => commitClipboardRestoreDelay(Number(e.target.value))}
                className="flex-1"
              />
              <span className="w-16 text-right font-mono text-[12px] tabular-nums text-fg-hi">
                {t('text_output.delivery.clipboard_restore_delay.value', {
                  milliseconds: clipboardRestoreDelay,
                })}
              </span>
            </div>
          </Row>
          <Row
            label={t('text_output.delivery.keystroke_fallback.label')}
            description={t('text_output.delivery.keystroke_fallback.description')}
          >
            <Toggle checked={Boolean(to.keystroke_fallback)} onChange={(v) => void update('keystroke_fallback', v)} />
          </Row>
        </Card>

        <Card title={t('text_output.formatting.title')}>
          <Row
            label={t('text_output.formatting.auto_capitalise.label')}
            description={t('text_output.formatting.auto_capitalise.description')}
          >
            <Toggle checked={Boolean(to.auto_capitalise)} onChange={(v) => void update('auto_capitalise', v)} />
          </Row>
          <Row
            label={t('text_output.formatting.trim_trailing_space.label')}
            description={t('text_output.formatting.trim_trailing_space.description')}
          >
            <Toggle checked={Boolean(to.trim_trailing_space)} onChange={(v) => void update('trim_trailing_space', v)} />
          </Row>
          <Row
            label={t('text_output.formatting.auto_space_on_paste.label')}
            description={t('text_output.formatting.auto_space_on_paste.description')}
          >
            <Toggle
              checked={Boolean(to.auto_space_on_paste ?? true)}
              onChange={(v) => void update('auto_space_on_paste', v)}
            />
          </Row>
          <Row
            label={t('text_output.formatting.numbers_as_digits.label')}
            description={t('text_output.formatting.numbers_as_digits.description')}
          >
            <Toggle
              checked={Boolean(to.numbers_as_digits ?? true)}
              onChange={(v) => void update('numbers_as_digits', v)}
            />
          </Row>
          <Row
            label={t('text_output.formatting.smart_rewrite.label')}
            description={t('text_output.formatting.smart_rewrite.description')}
          >
            <Toggle
              checked={Boolean(to.smart_rewrite ?? false)}
              onChange={(v) => void update('smart_rewrite', v)}
            />
          </Row>
          <Row
            label={t('text_output.formatting.shift_to_auto_send.label')}
            description={t('text_output.formatting.shift_to_auto_send.description')}
          >
            <Toggle checked={Boolean(to.shift_to_auto_send)} onChange={(v) => void update('shift_to_auto_send', v)} />
          </Row>
        </Card>
      </div>
    </div>
  );
}
