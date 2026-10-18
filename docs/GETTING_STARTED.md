# Getting Started with Utterbar

[Download stable](https://github.com/badoomtch/Utterbar-App/releases/latest)
· [Beta builds and release notes](https://github.com/badoomtch/Utterbar-App/releases)
· [Help](TROUBLESHOOTING.md)

## Install and Set Up

1. Download `Utterbar_<version>_x64-setup.exe` from a published GitHub Release.
   Source-code archives are for developers, not ready-to-run installers.
2. Install on 64-bit Windows 10 or 11. The interface requires WebView2.
3. Complete onboarding, select your microphone, and install a speech model.
   Model downloads need an internet connection.
4. Test a sentence in Notepad before dictating into important documents.

Unsigned installers may trigger SmartScreen. Check the repository, release and
filename before deciding whether to continue. Avoid unknown mirrors.

## Record a Dictation

Click in the field where you want the text. With default shortcuts:

| Shortcut       | Action                               |
| -------------- | ------------------------------------ |
| Ctrl + Space   | Start recording; press again to stop |
| F6             | Hold to record, release to stop      |
| Ctrl + Alt + X | Cancel recording                     |
| Ctrl + Alt + M | Cycle mode                           |
| Ctrl + Alt + , | Open settings                        |

Existing installations may have different bindings. **Settings → Shortcuts**
is the source of truth for your machine.

The floating bar shows recording and processing state. Stopping recording is
not finishing transcription: audio-tail capture and model processing may
continue briefly. Keep the target field focused until delivery. If paste fails,
use the bar's copy-last-result action and paste manually.

## Choose a Model and Language

In **Settings → Models**, choose Parakeet for English or an installed
multilingual Whisper model for other languages. Set the Whisper language or
use Auto-detect. Changing interface language does not change the speech model.

Larger models may improve recognition for some audio but use more memory and
take longer. Compare the same sentence on your machine rather than assuming
the largest model is always best.

**Keep model loaded forever** avoids unloading after each dictation and is the
current default for new settings. Existing saved preferences still apply.
Startup loading still takes time; this trades memory for less repeated loading.

## Adjust the Two Delays Separately

**Settings → Audio → End-of-speech capture** controls how long capture
continues after stopping. It protects final words. Lower values may feel faster
but risk cutting speech off. Active speech can extend the tail beyond the base
setting.

**Settings → Text Output → Clipboard restore delay** controls when the previous
clipboard returns after pasting. It is not recording time. A slower editor may
need a longer delay to read the temporary transcript. Restoration also depends
on the formats supplied by the source app; report repeatable failures.

## Improve Recognition

- Use the correct microphone and a clear signal without clipping or background noise.
- Choose a model supporting the language you speak.
- Add names and specialised terms to Vocabulary. Use replacements for
  repeatable errors, not broad fragments of common words.
- Compare raw recognition with your selected mode's formatting. These are
  separate processing steps.
- Report the exact phrase, output, model, language and repeatability when
  words are missing, rather than changing settings at random.

## Meetings, Files and Local Data

Meeting mode handles longer sessions in chunks. Choose microphone, system
audio or mixed capture to match what you want recorded. The Files page
processes existing recordings and exports text and subtitles.

Preferences and history live under `%APPDATA%\Utterbar\`. Audio retention is
configurable in **Settings → Privacy**. Get consent before recording others,
and review retention before handling confidential material.

## Interface Languages

English, Spanish, French, German, Italian, Portuguese and Dutch are selectable
in **Settings → General → App language**. Some surfaces still use English,
and review translations may contain mistakes. Send the screen, current wording
and suggested correction when reporting a translation issue.

## Updates and Support

Select stable or beta in **Settings → Updates**. Prereleases are testing builds;
the latest ordinary release is the default download. Older website-updater
installations require a manual update from GitHub.

Read [Troubleshooting](TROUBLESHOOTING.md), use
[GitHub Issues](https://github.com/badoomtch/Utterbar-App/issues/new/choose), or
email **[jay@jayparry.me](mailto:jay@jayparry.me)**.
