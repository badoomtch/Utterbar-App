import { useEffect, useState } from 'react';
import { ChevronDown, ChevronRight, ExternalLink } from 'lucide-react';
import { open as openShell } from '@tauri-apps/plugin-shell';
import { getVersion } from '@tauri-apps/api/app';

import { PageHeader } from '@/components/common/PageHeader';
import { Card } from '@/components/common/Card';
import { UtterbarLogoMark } from '@/components/bar/UtterbarLogoMark';
import { UtterbarWordmark } from '@/components/bar/UtterbarWordmark';
import { THIRD_PARTY_LICENCES_MD } from '@/data/thirdPartyLicences';

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
          {/* Matches the website footer's wording exactly:
              "© {year} Utterbar · Built by Ddraig Dev · Built in Wales".
              Single source of truth so the app and the marketing site
              never drift apart on attribution. */}
          <div className="text-[13px] text-muted">
            © {new Date().getFullYear()} Utterbar · Built by{' '}
            <button
              onClick={() => void openShell('https://ddraig.dev').catch(() => undefined)}
              className="group inline-flex items-center gap-1 text-fg-hi hover:text-accent"
            >
              Ddraig Dev
              <ExternalLink className="h-3 w-3 opacity-60 group-hover:opacity-100" />
            </button>{' '}
            · Built in Wales
          </div>
          <button
            onClick={() => void openShell('https://utterbar.com').catch(() => undefined)}
            className="font-mono text-[11px] text-fg-dim transition-colors hover:text-accent"
          >
            utterbar.com
          </button>
        </div>
      </Card>

      <ThirdPartyLicencesCard />
    </div>
  );
}

/**
 * Third-party licences surface — required attribution for the
 * CC-BY-4.0 models in the install (Parakeet TDT v3, NVIDIA NeMo
 * TitaNet small). Renders the bundled markdown copy as
 * preformatted plain text inside a collapsed Card; the user
 * expands it to read or copy.
 *
 * Plain `<pre>` rendering on purpose:
 *   - no Markdown renderer dependency (one less attack surface
 *     post-XSS hardening in the v0.14.13 patch)
 *   - exact byte-for-byte match with the canonical
 *     /THIRD_PARTY_LICENCES.md file
 *   - mono font is appropriate for licence text
 */
function ThirdPartyLicencesCard() {
  const [expanded, setExpanded] = useState(false);
  return (
    <Card className="mt-4">
      <button
        type="button"
        onClick={() => setExpanded((x) => !x)}
        className="flex w-full items-center justify-between gap-3 text-left"
      >
        <div>
          <div className="text-sm font-medium text-fg">Third-party licences</div>
          <div className="mt-1 text-[12px] text-muted">
            Open-source libraries and models bundled with Utterbar.
          </div>
        </div>
        {expanded ? (
          <ChevronDown className="h-4 w-4 text-muted" />
        ) : (
          <ChevronRight className="h-4 w-4 text-muted" />
        )}
      </button>
      {expanded && (
        <pre
          className="mt-4 max-h-[480px] overflow-auto rounded-input bg-bg-1 p-4 font-mono text-[11.5px] leading-relaxed text-fg whitespace-pre-wrap"
          // Plain-text render. No dangerouslySetInnerHTML, no
          // Markdown parser. The backtick template literal in
          // thirdPartyLicences.ts is the entire trust boundary.
        >
          {THIRD_PARTY_LICENCES_MD}
        </pre>
      )}
    </Card>
  );
}
