import React from 'react';
import ReactDOM from 'react-dom/client';
import { HashRouter, Route, Routes } from 'react-router-dom';

import './styles/globals.css';
// Side-effect import: initialises i18next with the bundled English
// resources before any component calls `useTranslation()`. Must precede
// the route imports so the first render finds the namespace registered.
import '@/i18n';

import { AppBootstrap } from '@/routes/AppBootstrap';
import { FloatingBar } from '@/routes/FloatingBar';
import { Onboarding } from '@/routes/Onboarding';
import { MainShell } from '@/routes/MainShell';
import { VocabAdd } from '@/routes/VocabAdd';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <HashRouter>
      <AppBootstrap>
        <Routes>
          <Route path="/floating" element={<FloatingBar />} />
          <Route path="/onboarding/*" element={<Onboarding />} />
          <Route path="/vocab-add" element={<VocabAdd />} />
          <Route path="/*" element={<MainShell />} />
        </Routes>
      </AppBootstrap>
    </HashRouter>
  </React.StrictMode>,
);
