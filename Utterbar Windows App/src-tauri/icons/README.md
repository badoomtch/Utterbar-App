# Icons

Vector sources live here; the rasters are generated, not hand-authored.

## Sources

- `mark.svg` — the full Utterbar mark (pill with glow + waveform). Used for the
  taskbar / installer icon.
- `wave.svg` — just the waveform bars. Used for the system tray icon so it reads
  cleanly at 16×16 against any taskbar theme.
- `lockup.svg` — "Utterbar" horizontal wordmark for onboarding / About page.

## Generated rasters

Run `python scripts/generate-icon.py` (needs `resvg_py` + `Pillow`) to regenerate:

- `source.png` — the mark on a transparent background. **Feed this into
  `pnpm tauri icon src-tauri/icons/source.png`** to fan out `icon.ico`,
  `32x32.png`, `128x128.png`, `128x128@2x.png` and all the `Square*Logo.png`
  store tiles. This is what the taskbar, Start menu, and installer show — no
  dark rounded-square tile behind it.
- `source-square.png` — the mark on a dark rounded-square. Retained only because
  Microsoft Store tile assets (`StoreLogo.png`, `Square*Logo.png`) must be opaque
  on some surfaces. Not fed into `tauri icon` anymore.
- `tray.png` — the waveform on transparent, 256×256. Loaded directly by Tauri
  (see `tauri.conf.json` → `trayIcon.iconPath`).

## Regenerate everything

```powershell
python scripts/generate-icon.py
pnpm tauri icon src-tauri/icons/source.png
```

Keep the vector sources square-ish with enough padding to avoid clipping at small
sizes, and re-run the commands above whenever the marks change.
