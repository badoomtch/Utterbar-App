import { useTranslation } from 'react-i18next';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { useSettings } from '@/state/settings';

export function TextOutputSettings() {
  const { t } = useTranslation('settings');
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  if (!s) return null;

  const update = (key: string, value: unknown) =>
    patch({ text_output: { ...s.text_output, [key]: value } });

  // Renamed from `t` to avoid shadowing the i18next translator above. The
  // structural settings field shape is unchanged — every row still reads
  // `to.<field>` and writes via `update('<field>', v)` against the
  // `text_output.<field>` slot on Settings.
  const to = s.text_output as Record<string, boolean>;

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader title={t('text_output.title')} subtitle={t('text_output.subtitle')} />
      <div className="mt-6 space-y-4">
        <Card title={t('text_output.delivery.title')}>
          <Row
            label={t('text_output.delivery.paste_automatically.label')}
            description={t('text_output.delivery.paste_automatically.description')}
          >
            <Toggle checked={to.paste_automatically} onChange={(v) => void update('paste_automatically', v)} />
          </Row>
          <Row
            label={t('text_output.delivery.restore_clipboard.label')}
            description={t('text_output.delivery.restore_clipboard.description')}
          >
            <Toggle checked={to.restore_clipboard} onChange={(v) => void update('restore_clipboard', v)} />
          </Row>
          <Row
            label={t('text_output.delivery.keystroke_fallback.label')}
            description={t('text_output.delivery.keystroke_fallback.description')}
          >
            <Toggle checked={to.keystroke_fallback} onChange={(v) => void update('keystroke_fallback', v)} />
          </Row>
        </Card>

        <Card title={t('text_output.formatting.title')}>
          <Row
            label={t('text_output.formatting.auto_capitalise.label')}
            description={t('text_output.formatting.auto_capitalise.description')}
          >
            <Toggle checked={to.auto_capitalise} onChange={(v) => void update('auto_capitalise', v)} />
          </Row>
          <Row
            label={t('text_output.formatting.trim_trailing_space.label')}
            description={t('text_output.formatting.trim_trailing_space.description')}
          >
            <Toggle checked={to.trim_trailing_space} onChange={(v) => void update('trim_trailing_space', v)} />
          </Row>
          <Row
            label={t('text_output.formatting.auto_space_on_paste.label')}
            description={t('text_output.formatting.auto_space_on_paste.description')}
          >
            <Toggle
              checked={to.auto_space_on_paste ?? true}
              onChange={(v) => void update('auto_space_on_paste', v)}
            />
          </Row>
          <Row
            label={t('text_output.formatting.numbers_as_digits.label')}
            description={t('text_output.formatting.numbers_as_digits.description')}
          >
            <Toggle
              checked={to.numbers_as_digits ?? true}
              onChange={(v) => void update('numbers_as_digits', v)}
            />
          </Row>
          <Row
            label={t('text_output.formatting.smart_rewrite.label')}
            description={t('text_output.formatting.smart_rewrite.description')}
          >
            <Toggle
              checked={to.smart_rewrite ?? false}
              onChange={(v) => void update('smart_rewrite', v)}
            />
          </Row>
          <Row
            label={t('text_output.formatting.restore_punctuation.label')}
            description={t('text_output.formatting.restore_punctuation.description')}
          >
            <Toggle
              checked={to.restore_punctuation ?? false}
              onChange={(v) => void update('restore_punctuation', v)}
            />
          </Row>
          <Row
            label={t('text_output.formatting.shift_to_auto_send.label')}
            description={t('text_output.formatting.shift_to_auto_send.description')}
          >
            <Toggle checked={to.shift_to_auto_send} onChange={(v) => void update('shift_to_auto_send', v)} />
          </Row>
        </Card>
      </div>
    </div>
  );
}
