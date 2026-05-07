import React, { Suspense, lazy } from 'react';
import ReactDOM from 'react-dom/client';
import { HashRouter, Route, Routes } from 'react-router-dom';

import './styles/globals.css';
// Side-effect import: initialises i18next with the bundled English
// resources before any component calls `useTranslation()`. Must precede
// the route imports so the first render finds the namespace registered.
import '@/i18n';

import { AppBootstrap } from '@/routes/AppBootstrap';
// FloatingBar is the only route loaded eagerly at the top level. The
// floating window is the latency-critical path — it's the first window
// Tauri creates at boot, it's the surface the user sees the moment the
// hotkey fires, and any extra round-trip to fetch a chunk on first
// render would push perceptible latency into the dictation flow. The
// other three top-level routes (MainShell, Onboarding, VocabAdd) are
// hosted in separate Tauri windows that the user opens deliberately;
// paying a tiny chunk-fetch on those windows' first render is fine and
// trims the floating bar's initial bundle measurably (the settings
// panes, history/files/meetings views, and onboarding flow no longer
// parse on every window's boot).
import { FloatingBar } from '@/routes/FloatingBar';

const MainShell = lazy(() =>
  import('@/routes/MainShell').then((m) => ({ default: m.MainShell })),
);
const Onboarding = lazy(() =>
  import('@/routes/Onboarding').then((m) => ({ default: m.Onboarding })),
);
const VocabAdd = lazy(() =>
  import('@/routes/VocabAdd').then((m) => ({ default: m.VocabAdd })),
);

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <HashRouter>
      <AppBootstrap>
        <Suspense fallback={null}>
          <Routes>
            <Route path="/floating" element={<FloatingBar />} />
            <Route path="/onboarding/*" element={<Onboarding />} />
            <Route path="/vocab-add" element={<VocabAdd />} />
            <Route path="/*" element={<MainShell />} />
          </Routes>
        </Suspense>
      </AppBootstrap>
    </HashRouter>
  </React.StrictMode>,
);
