import React from 'react';
import ReactDOM from 'react-dom/client';
import { HashRouter, Route, Routes } from 'react-router-dom';

import './styles/globals.css';

import { AppBootstrap } from '@/routes/AppBootstrap';
import { FloatingBar } from '@/routes/FloatingBar';
import { Onboarding } from '@/routes/Onboarding';
import { MainShell } from '@/routes/MainShell';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <HashRouter>
      <AppBootstrap>
        <Routes>
          <Route path="/floating" element={<FloatingBar />} />
          <Route path="/onboarding/*" element={<Onboarding />} />
          <Route path="/*" element={<MainShell />} />
        </Routes>
      </AppBootstrap>
    </HashRouter>
  </React.StrictMode>,
);
