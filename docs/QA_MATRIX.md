# Utterbar — QA matrix

Manual QA to run before cutting a release. Each row lists the steps, the expected result, and the affected subsystems. The goal is coverage breadth, not exhaustive cases — automated tests handle the permutations.

## Reference rig

- Windows 11 Pro, latest cumulative update
- 16 GB RAM, mid-range CPU, discrete GPU optional
- A second monitor to exercise multi-display behaviour

## Matrix

| # | Scenario | Steps | Expected | Subsystems |
|---|---------|------|----------|-----------|
| 1 | First-run onboarding | Delete `%APPDATA%\Utterbar`, launch | Onboarding window appears; every step navigates forward and back | Windows, Settings, Onboarding |
| 2 | Model install | Settings → Models → Install Parakeet EN | Progress bar advances; "Installed" after completion; model file exists on disk | Models, Download |
| 3 | Mic selection | Settings → Audio → pick non-default device → Test | Meter animates; switching back to default still works | Audio |
| 4 | Dictation (no LLM) | Press Toggle hotkey → speak → press again | Floating bar: idle → listening → processing → success → idle; text appears in active app | Hotkeys, Audio, ASR, TextInject |
| 5 | Cancel recording | Start recording → press Cancel hotkey | Phase returns to idle without inserting text | Hotkeys, Dictation |
| 6 | Mode switch | Floating bar → mode menu → pick Email | Next dictation tagged Email in History; processed text formatted accordingly | Modes, History |
| 7 | Vocabulary | Add word "Kubernetes" → dictate containing it | Word appears verbatim | Vocabulary |
| 8 | Text replacement | Add `btw → by the way` → dictate "btw" | Replacement applied | Vocabulary, Replacements |
| 9 | Paste fallback | Focus an elevated (UAC) process → dictate | Keystroke fallback engages; text lands | TextInject |
| 10 | Clipboard restore | Copy "apple" → dictate with clipboard restore on | Dictated text pasted, then clipboard reverts to "apple" within 2 s | TextInject |
| 11 | Context capture | Enable all context toggles → dictate in Outlook | History entry shows app_exe/title, clipboard, selection in Context inspection | Context, Privacy |
| 12 | Super mode | Active mode = Super → dictate with text selected | Processed output references the selection | Modes, Context, LLM |
| 13 | Reprocess | History → pick entry → select different mode → Run | New entry appended with correct mode tag | History, Modes |
| 14 | History filter | Type partial transcript in search box | Debounced filter returns only matching rows | History |
| 15 | Analytics | Dictate several times → Dashboard | Words today, dictations today, time saved all increment | Analytics |
| 16 | File transcription | Drag an MP3 onto Files | Queue row appears; progress advances | Files |
| 17 | Tray behaviour | Close main window | App stays in tray; left-click tray re-opens | Tray, Windows |
| 18 | Launch on login | Toggle on → reboot | Utterbar auto-starts with bar visible | Startup, Settings |
| 19 | Multi-monitor | Drag floating bar to second monitor | Bar remembers monitor across relaunch | Windows, DPI |
| 20 | High-DPI | Switch display scaling to 150 % | No blurry text, waveform stays crisp | Windows |
| 21 | Reduce motion | Settings → General → Reduce motion on | Floating bar transitions become instant fades | Motion, Settings |
| 22 | Theme switch | Cycle Obsidian / Graphite / Paper | Colors update immediately without restart | Settings, Tokens |
| 23 | Accent switch | Pick each accent | Accent affects bar glow, buttons, highlights | Settings, Tokens |
| 24 | Privacy inspection | Dictate → History → expand Context inspection | Shows exact captured payload (or empty when disabled) | Context, History |
| 25 | Telemetry | Confirm telemetry off → monitor network with Proc Monitor | No outbound requests during dictation | Privacy |
| 26 | Updater | Mock a newer version → open main window | Update toast appears; Install on restart works | Updater |
| 27 | Reset | Settings → Advanced → Reset | Settings back to defaults; modes/history preserved | Settings |
| 28 | Crash-safe settings | Kill process mid-save | `settings.json` remains valid; `.tmp` cleaned on next launch | Settings |
| 29 | Diagnostics | Settings → Advanced → Open logs | File explorer opens at `%APPDATA%\Utterbar\logs\` | Diagnostics |
| 30 | Keyboard-only navigation | Tab through main window with no mouse | Every primary action reachable; focus rings visible | Accessibility |

## Sign-off

Release can ship when rows 1–20 all pass on the reference rig, and no row has a P0/P1 bug open.
