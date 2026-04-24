import { useEffect, useState } from 'react';
import { ExternalLink } from 'lucide-react';
import { open as openShell } from '@tauri-apps/plugin-shell';
import { getVersion } from '@tauri-apps/api/app';

import { PageHeader } from '@/components/common/PageHeader';
import { Card } from '@/components/common/Card';
import { UtterbarLogoMark } from '@/components/bar/UtterbarLogoMark';
import { UtterbarWordmark } from '@/components/bar/UtterbarWordmark';

export function AboutSettings() {
  const [version, setVersion] = useState<string>('');
  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => undefined);
  }, []);

  return (
    <div className="mx-auto max-w-2xl p-8">
      <PageHeader title="About" />
      <Card className="mt-6">
        <div className="flex items-center gap-4">
          <UtterbarLogoMark size={56} />
          <div>
            <UtterbarWordmark size={32} />
            <div className="mt-1 font-mono text-[11px] text-fg-dim">
              {version ? `v${version}` : 'v…'}
            </div>
          </div>
        </div>
        <div className="mt-5 text-[13px] leading-relaxed text-muted max-w-[58ch]">
          Local-only dictation for Windows. All audio and text processing is performed on this
          machine. No cloud services are contacted for transcription or post-processing.
        </div>

        <div className="mt-6 flex items-center justify-between border-t border-border-soft/50 pt-4">
          <button
            onClick={() => void openShell('https://jayparry.me').catch(() => undefined)}
            className="group inline-flex items-center gap-1.5 text-[13px] text-muted transition-colors hover:text-fg-hi"
          >
            App by <span className="text-fg-hi group-hover:text-accent">Jay Parry</span>
            <ExternalLink className="h-3 w-3 opacity-60 group-hover:opacity-100" />
          </button>
          <button
            onClick={() => void openShell('https://utterbar.com').catch(() => undefined)}
            className="font-mono text-[11px] text-fg-dim transition-colors hover:text-accent"
          >
            utterbar.com
          </button>
        </div>
      </Card>
    </div>
  );
}
