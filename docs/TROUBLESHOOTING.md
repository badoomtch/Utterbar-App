# Troubleshooting

Support: **[jay@jayparry.me](mailto:jay@jayparry.me)** or
[GitHub Issues](https://github.com/badoomtch/Utterbar-App/issues/new/choose).
Report vulnerabilities [privately](../SECURITY.md).

## Text Does Not Paste

1. Check whether the result exists in history. No transcript means the problem
   is earlier than delivery.
2. Compare the failing app with Notepad. Click in the target field before
   recording and keep it focused through processing.
3. Check automatic paste under **Settings → Text Output**. Try a longer
   **Clipboard restore delay** for editors that read the clipboard slowly.
4. Check whether the target runs as administrator. Windows can block input
   from an app at a lower privilege level. Prefer running the target normally;
   do not routinely elevate Utterbar as a workaround.
5. Recover text with the floating bar's copy-last-result action.

Report the target app/version, whether it is elevated, the restore-delay value,
and whether the same phrase pastes into Notepad. Keep private clipboard data
out of public reports.

## The Previous Clipboard Is Not Restored

Note whether it contained text, an image, files or an app-specific format, and
which app copied it. Check the restore delay. Copying something new during
dictation may legitimately change the clipboard too. Reproduce with harmless
sample content, not confidential data.

## Missing Words or Cut-off Sentences

Check the input device and model language first. Increase **Settings → Audio →
End-of-speech delay** for cut-off final words. Missing words in the middle are
not explained by that setting alone: provide a repeatable phrase and compare
models or input sources.

Include the spoken phrase, output, model, language, mode and input source.
Mention vocabulary replacements and formatting. Do not post private audio.

## Slow First Dictation

Models must load into memory. Check the loading feedback and **Settings →
Models → Keep model loaded forever** if you have enough memory. Larger models
and slower hardware still take longer. If a shortcut fails, use the visible
controls rather than repeatedly pressing record during processing.

## No Audio or Wrong Language

Check Windows microphone permissions and **Settings → Audio**. Select the
actual microphone rather than an unused headset input. System audio records
computer playback, not necessarily your microphone. Non-English speech needs
a multilingual model; changing interface language is not enough.

## Media and Call Muting

Media pause, conferencing-output mute and microphone mute are different.
Output mute changes what you hear, not what call participants hear.
Experimental mic mute supports Teams and Discord hotkeys only: configure the
call app's shortcut and check its own mute indicator. Use the app's mute button
or hardware mute when you need a guaranteed result.

## Updates

Check the channel and [published releases](https://github.com/badoomtch/Utterbar-App/releases).
Stable ignores prereleases. Older website-updater builds need a manual GitHub
installation. The installer is `Utterbar_<version>_x64-setup.exe`; source ZIPs
are not installers.

## Crashes and Logs

Logs are under `%APPDATA%\Utterbar\logs\`. Inspect them for private text,
personal information and file paths before sharing. Email sensitive diagnostics
to **jay@jayparry.me**, not a public issue.

Include your Utterbar and Windows versions, model, GPU/provider setting, input
device, reproduction steps and approximate crash time. Say whether it repeats
after restarting or with another model.

Do not delete `%APPDATA%\Utterbar\` without backing up what you need: it can
contain history, settings, models and recordings.
