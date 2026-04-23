import { useEffect, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { Download } from 'lucide-react';

import { Button } from '@/components/common/Button';
import { invoke } from '@/lib/ipc';
import { springs } from '@/design/motion';
import { useSettings } from '@/state/settings';

type UpdateInfo = { current: string; available: string | null; notes: string | null };

/**
 * Tiny bottom-right toast that surfaces new versions without nagging.
 *
 * Per DESIGN_SYSTEM.md the toast sits on bg-1 (matching card elevation), has
 * no drop shadow (depth is carried by tonal lift over the app background),
 * and limits its cyan moment to a single icon + the CTA pill — we stay well
 * inside the §1.6 cyan budget.
 */
export function UpdateToast() {
  const [info, setInfo] = useState<UpdateInfo | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const [installing, setInstalling] = useState(false);
  const autoCheck = useSettings((x) => x.data?.general.check_updates_automatically ?? true);

  useEffect(() => {
    if (!autoCheck) return;
    void invoke<UpdateInfo>('check_update')
      .then((u) => {
        if (u.available) setInfo(u);
      })
      .catch(() => undefined);
  }, [autoCheck]);

  const install = async () => {
    setInstalling(true);
    try {
      await invoke('install_update');
    } finally {
      setInstalling(false);
    }
  };

  const show = info && info.available && !dismissed;

  return (
    <AnimatePresence>
      {show && (
        <motion.div
          key="update-toast"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 10 }}
          transition={springs.glide}
          className="fixed bottom-5 right-5 flex w-80 items-start gap-3 rounded-card bg-bg-1 px-4 py-3"
        >
          <Download className="mt-0.5 h-4 w-4 text-accent" />
          <div className="flex-1">
            <div className="text-[13px] font-medium text-fg-hi">Update available</div>
            <div className="mt-0.5 font-mono text-[11px] text-muted">
              {info!.current} → {info!.available}
            </div>
            <div className="mt-2 flex items-center gap-2">
              <Button
                variant="cta"
                size="sm"
                onClick={() => void install()}
                disabled={installing}
              >
                {installing ? 'Installing…' : 'Install on restart'}
              </Button>
              <button
                onClick={() => setDismissed(true)}
                className="text-[12px] text-muted transition-colors hover:text-fg-hi"
              >
                Later
              </button>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
