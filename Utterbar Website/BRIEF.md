# Utterbar — Website Brief

> **Scope note (2026-04-24):** this file is authoritative for **product positioning, brand voice, and marketing copy tone** — the things that describe what the product is and how to talk about it. It is NOT authoritative for: the current shipping status, tech stack details, visual tokens, or any implementation-level "we have this / we don't have this" claim. Those live elsewhere:
>
> - Architecture, stack, and the shipping state of endpoints → [../MONOREPO.md](../MONOREPO.md)
> - Visual language, tokens, components → [./DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md)
> - Repo-wide working rules → [../CLAUDE.md](../CLAUDE.md)
>
> If something in this file contradicts one of those, they win.
>
> The domain is `utterbar.com`, not `utterbar.app` (an earlier working name). The product ships — the site has live Stripe Checkout and real downloads — any claim in this file to the contrary (sections 2 and 3.2 are the most stale) is historical and kept only because the voice / copy sections around them are still useful.

---

## 1. What Utterbar is

Utterbar is a premium dictation utility for Windows. It sits as a small floating pill on the user's desktop, listens when they hit a global hotkey (default `Ctrl+Space`), transcribes their speech with NVIDIA Parakeet running 100% on-device, and injects the text straight into whichever app is focused — Chrome, VS Code, Notion, Slack, anywhere.

Key facts to anchor every page:

- **Local-only.** No cloud. Audio never leaves the machine. We say this loudly and repeatedly.
- **Windows-first.** macOS users have Superwhisper; Utterbar is built for Windows from day one (no Electron, no WSL, no Python shim in the hot path — it's Tauri + Rust).
- **Fast.** Parakeet TDT (0.6B v2) runs on CPU or GPU; real sentences finish transcribing in well under a second on a modern laptop. DirectML lights up any DX12 GPU (NVIDIA / AMD / Intel); CUDA path for NVIDIA-only users.
- **Works with every app.** Anything that accepts keystrokes accepts Utterbar output. For UIPI-locked / elevated targets we auto-fall back to per-character keystroke injection.
- **Modes.** Voice-to-text, Message, Email, Note, Meeting, Super — each swaps model, post-processing prompt, and formatting rules. Custom modes on Pro.
- **Capture anything that plays on your PC.** Mic, system-audio (WASAPI loopback, no virtual cable needed), or both summed. Good for transcribing the other side of a Teams / Zoom / Discord call.
- **Vocabulary.** Teach it names, acronyms, jargon. Those words stop getting mis-transcribed.
- **Privacy switches** for clipboard / selection / active-app context. Off by default (active-app on by default, the other two off) for the paranoid ones.

Elevator pitch (use verbatim on hero):

> Dictation that stays on your PC. Utterbar transcribes your voice into any Windows app — instantly, offline, and without sending a single byte to the cloud.

---

## 2. Status — HISTORICAL, do not follow

The section that used to live here described a pre-release waiting-list phase: no downloads, no Stripe, no login, no user accounts. **None of that is true any more.** As of v0.6 the site has:

- A real `/download` page serving the latest NSIS installer from R2 / `dl.utterbar.com`.
- Live Stripe Checkout at `£39.99` GBP one-time for Pro (3-device cap).
- Magic-link auth for customer accounts at `/sign-in` → `/account`.
- Admin surfaces at `/admin` (release publishing, licence management).
- Licence-issuance backend wired to a Stripe webhook.
- Licence-activation API that the desktop app calls over HTTPS.

The shipping picture lives in [../MONOREPO.md §3–§5](../MONOREPO.md). Use that, not this section. This heading is kept as a deliberate landmark so future readers notice the change — the voice / copy sections below it are still useful and intentionally preserved.

---

## 3. Brand

### 3.1 Name

`Utterbar` — one word, capital U, lowercase rest. Never `UtterBar`, never `utter bar`. The utterance bar. The thing you utter into.

The floating pill in the app is called "the Utterbar" (definite article) in docs and help content. On the website, lean on "Utterbar" as a proper noun.

### 3.2 Palette — see DESIGN_SYSTEM.md

The exact hex tokens, gradients, and theme rules live in [./DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md) and are the authoritative source. Earlier versions of this file inlined a `#22D3EE` / cyan-400 palette and an `Inter` + `JetBrains Mono` font pair — both have since drifted from the shipped look. Do not copy values from this section; always pull from DESIGN_SYSTEM.md (or the live `src/styles/tokens.css` files in the app and the website's `resources/css/app.css`).

What's still authoritative from this brief at the identity level:

- Electric cyan identity colour. Not navy, not enterprise-SaaS blue. Tape-deck VU-meter glow.
- Dark by default. Cyan is reserved for action, focus, and live-listening cues — never a body-copy tint.
- The accent gradient used in the logo waveform is canonical and lives in the app's SVG assets (`src-tauri/icons/mark.svg` etc.).

### 3.3 Typography — see DESIGN_SYSTEM.md

Font families, weights, and type scale are in [./DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md). The authoritative fonts are Instrument Sans (body/display) and Geist Mono (mono) — earlier versions of this brief named Inter + JetBrains Mono and are superseded.

### 3.4 Logo

The logo is a cyan gradient **pill** with a glowing LED (the "light") and a waveform glyph below it. The app ships three canonical SVG sources — reuse those, do not redraw:

- `src-tauri/icons/mark.svg` — the full pill + light + waveform. Taskbar, installer, Start menu (rasterised via `scripts/generate-icon.py` → `source.png` → `pnpm tauri icon`).
- `src-tauri/icons/wave.svg` — waveform bars only. System tray icon.
- `src-tauri/icons/lockup.svg` — horizontal pill + "Utterbar" wordmark. Onboarding / About / site nav.

Cyan gradient used in-app: `#00f6ff → #00a3ff` (vertical). If the site's `--accent` (`#22D3EE`) has visibly drifted from what the pill glows with, sync to the in-app gradient for hero visuals — the marketing chrome can stay on the `#22D3EE` token but the pill mock must match the real app.

Lockups:
- **Horizontal** (marketing default): pill-mark + wordmark, 32px tall at minimum.
- **Vertical** (social avatars, favicons): pill-mark alone, square padded.
- **Mono-inverse**: plain white pill for light photographic backgrounds — we won't use it on-site but include it for press.

Clear space around the logo = the height of the LED dot on all four sides.

### 3.5 Iconography

Use `lucide` (matches what the app uses). Stroke width 1.5, size 18–20 in body, 24 in features, 32 in the hero quick-stats strip. Never mix icon libraries. Never use emoji as icons in marketing — only in the occasional in-context quote.

### 3.6 Tone of voice

Confident, specific, minimal. No marketing fluff. Three rules:

1. **Lead with the concrete thing.** "Your voice stays on your machine." not "Privacy-first AI-powered workflow".
2. **Name the competition when it sharpens a sentence.** Superwhisper is the Mac equivalent; saying so saves a paragraph.
3. **Short sentences on hero + features. Longer paragraphs are fine on docs/FAQ, but headlines are max 9 words.**

Banned words: _synergy, empower, revolutionise, game-changer, AI-powered (as a noun), seamless, unlock, frictionless, supercharge, next-gen_. If you catch yourself reaching for one, rewrite.

Preferred register: British English spelling (`colour`, `organise`, `licence` as noun, `license` as verb). The product ships worldwide but the founder is British, and the copy stays British.

---

## 4. Site architecture

```
/                           Home  (hero + features + how-it-works + pricing teaser + FAQ + waitlist)
/features                   Deep feature list with screenshots
/how-it-works               Two-minute explainer: hotkey → listen → transcribe → inject
/privacy                    The "why local" essay + threat model
/pricing                    Free / Pro / Team (CTAs all route to waitlist)
/changelog                  Markdown-rendered release notes (empty placeholder until 1.0)
/docs                       /docs/getting-started, /docs/modes, /docs/vocabulary, /docs/shortcuts,
                            /docs/privacy-controls, /docs/troubleshooting (stubbed — real content later)
/blog                       Optional; ship as a placeholder route only
/legal/privacy              Privacy policy
/legal/terms                Terms of service
/legal/eula                 End-user licence agreement (mirrors the app's LICENSE.txt)
/press                      Kit: logos, screenshots, one-liner, founder bio, email
/roadmap                    Public roadmap (Now / Next / Later columns)
/waitlist                   Standalone waitlist page (also embedded on / and /pricing)
/404                        Funny-but-on-brand 404
```

Navigation (top bar, left-to-right):  
`Features · How it works · Privacy · Pricing · Docs · [GitHub icon link]`

Right side of nav: a prominent `Join the waitlist` cyan button, always visible.

Footer columns:  
- **Product** — Features, How it works, Privacy, Pricing, Changelog, Roadmap  
- **Docs** — Getting started, Modes, Vocabulary, Shortcuts, Troubleshooting  
- **Company** — Press, Contact, Legal  
- **Social** — GitHub, X, (future) YouTube, Mastodon

Footer bottom strip: `© Utterbar · Built by Jay Parry · v—` with the version pulled dynamically from a shared `site.config.ts` constant so it tracks the app.

---

## 5. Page-by-page content

### 5.1 Home (`/`)

**Hero**

- Eyebrow: `Windows · Local-only · v0.3 preview` (monospaced, muted). Pulled dynamically from `config('site.app_version')` — current is 0.3.5.
- H1: **Dictation that stays on your PC.**
- Sub: _Utterbar transcribes your voice into any Windows app — instantly, offline, and without sending a single byte to the cloud._
- Primary CTA: `Join the waitlist` (cyan) — opens inline email capture.
- Secondary CTA: `See how it works` (ghost) — anchors to the how-it-works section.
- Visual: full-bleed animated mockup showing the floating pill expanding on `Ctrl+Space`, a short waveform scrubbing, then a transcribed line dropping into a Chrome window. Loop at ~6s. Reduced-motion users get a single still frame. Render this with Framer Motion + a looping Lottie OR a recorded `<video autoplay muted loop playsinline>` mp4 at ~1.5 MB — prefer the video.

**Trust strip** (just below hero)

Three chips side by side:

- 🛡️ Your audio never leaves your machine
- ⚡ Real-time transcription on CPU
- 🎯 Works in every app that accepts typing

Icons are lucide, not emoji — I put emoji here for quick reference only.

**Feature cards** (3 × 2 grid)

1. **Hit a key. Start talking.** — One global hotkey (default `Ctrl+Space`). Toggle, push-to-talk, or hold-to-talk — seven hotkey slots, all user-rebindable. Works over any app, on any monitor.
2. **Modes for every voice.** — Voice-to-text, Message, Email, Note, Meeting, Super. Switch with a click; post-processing prompt and formatting follow. Custom modes on Pro.
3. **Capture anything.** — Microphone, system audio (WASAPI loopback — transcribe the meeting, the podcast, the video), or both summed. No virtual-cable install, no weird driver.
4. **Teach it your words.** — Vocabulary list for names, acronyms, product terms. No more `Jay Parry` becoming `J parrie`.
5. **Pause your music automatically.** — Spotify, YouTube, Apple Music, VLC, anything registered with Windows SMTC stops when you hit the hotkey and resumes when you're done (only sessions Utterbar paused — nothing weird).
6. **No account. No cloud. No telemetry.** — Run it offline forever. The only network calls are model downloads from Hugging Face mirrors on first run, and you can see every byte.

**How it works** (three-step stepper)

Step 1 `Press Ctrl+Space` → the Utterbar wakes up, the LED pulses cyan.  
Step 2 `Speak naturally` → waveform moves, a local Parakeet model listens.  
Step 3 `Let go` → your text appears in the app under your cursor. No paste dialog, no copy step.

Under the stepper, a 30-second silent video/loop demonstrating the above — same footage style as the hero.

**Pricing teaser** (condensed version of `/pricing`)

Three columns with a single CTA each (`Join the waitlist`). Full details on pricing page. On home, show only the headline price and the top three bullets per tier.

**FAQ** (accordion, 8–10 Qs — content below)

**Final CTA**

Full-width cyan-glow band:  
H2: **Be the first to ship Utterbar 1.0.**  
Sub: _Waitlist members get early access and a 50% launch discount on Pro for life._  
Inline email field + `Join` button.

### 5.2 Features (`/features`)

Long scroll, one feature per section, image left / text right / alternating. For each feature, include:

- Big headline (H2)
- 2–3 paragraph explanation
- Bullet list of what's included
- Screenshot (use the real ones from the app — Jay will supply)

Sections, in order:

1. **One hotkey, every app** — seven global hotkey slots (ToggleRecording, PushToTalk, HoldToTalk, CancelRecording, CycleMode, OpenSettings, OpenHistory). Default: `Ctrl+Space` → ToggleRecording; the rest unbound so the user binds what they want. Each slot is Toggle / PushToTalk / HoldToTalk.
2. **Parakeet on your machine** — NVIDIA Parakeet TDT 0.6B v2, running through sherpa-onnx. Two quantisations ship:
   - **int8** (~460 MB) — CPU-capable, default, ships on every tier.
   - **fp16** (~1.16 GB) — GPU-only (DirectML or CUDA — CPU providers lack fp16 kernels). Pro tier.
   English-only today. Multilingual is on the roadmap.
3. **Modes** — swap model + post-processing prompt + formatting rules per context. Six built-ins: **Voice-to-text** (pass-through), **Message** (casual, template), **Email** (LLM: polite / well-formatted), **Note** (LLM: bullet structure), **Meeting** (pass-through, preserves linebreaks), **Super** (LLM with optional active-app / selection / clipboard context). Custom modes on Pro.
4. **Vocabulary & replacements** — hinted words (boost ASR weight), text replacements (regex or literal). Per-mode when you want, global when you don't.
5. **Post-processing** — deterministic (punctuation, capitalisation, filler removal, auto-space, sentence capitalisation) runs on every tier; optional LLM rewriting via on-device **llama.cpp** on Pro for tone / style rewrites (feature-gated at compile time — a deterministic fallback runs in builds without llama).
6. **Capture source** — Microphone, System audio (WASAPI loopback on the default render endpoint — no virtual-cable install), or Mixed (mic + system summed). Per-dictation noise suppression is a toggle: DC blocker → high-pass → adaptive gate, all in-house, zero-dep, calibrates from the quietest 20 % of the frame in the room.
7. **Text output** — paste-via-clipboard (restores your clipboard ~1.5 s after) or per-character keystroke injection. We try clipboard first, and if Windows blocks the `Ctrl+V` SendInput (UIPI / elevated targets / remote desktops) we fall back to keystrokes automatically. Optional "hold Shift to auto-send" presses Enter after the paste for Teams / Slack / Discord chat-box dictation.
8. **Privacy controls** — three context gates: active-app (on by default), selection (off), clipboard (off). Audio retention is off by default; turn it on and the WAV persists in `%APPDATA%\Utterbar\recordings\` with a 1–90 day retention slider (default 14). Pruning runs on every new dictation.
9. **Media auto-pause** — Windows SMTC integration. Spotify, YouTube, Apple Music, VLC — anything that advertises a media session gets paused when you start and resumed when you stop. Only resumes sessions Utterbar itself paused.
10. **File transcription** — Symphonia decoder supports WAV / MP3 / AAC / MP4 / MKV / Ogg / FLAC / PCM → mono 16 kHz → same ASR pipeline. Export `.txt` / `.srt` / `.vtt`. (Note: drag-drop UX polish is on the 1.0 punch-list — the backend path is wired.)
11. **History & reprocessing** — every dictation saved with its raw text, processed text, and (optional) raw audio. Reprocess later with a different mode or prompt.
12. **The bar** — a floating 420×48 pill, always-on-top, drag to reposition, double-click to maximise view, right-click to minimise. Visibility has three modes: Always / WhileRecording / Hidden. Size: Compact or Standard. Outline: on/off.
13. **Themes & accents** — three themes (Obsidian, Graphite, Paper) × six accents (Teal, Amber, Violet, Rose, Cyan, Lime). Cyan is the default; the brand stays cyan even if the user picks amber in-app.
14. **Accessibility** — global "Reduce motion" toggle kills CSS transitions and tames Framer springs across the app. Sound cues (synthesised in-app via WebAudio — no sample files) and Windows toast notifications are both user-togglable.
15. **GPU acceleration** — Advanced → GPU acceleration: **Auto** (default, picks DirectML on any DX12 GPU), **DirectML** (NVIDIA / AMD / Intel), **CUDA** (NVIDIA-only, fastest on supported cards), **CPU** (fallback). Lives on `Advanced` settings; model keep-warm (0–600 s) tunes cold-start latency.
16. **Native Windows integration** — tray icon (waveform glyph only), taskbar icon (full pill mark, transparent background), single-instance, auto-start option, proper NSIS + MSI installer, WebView2 via embedded bootstrapper, signed binaries coming.

### 5.3 How it works (`/how-it-works`)

Long-form technical explainer. Still marketing — not docs — but aimed at the reader who wants to know _how_ it works before they trust it.

Sections:

1. **The stack** — Tauri 2 webview + Rust core + sherpa-onnx (ONNX Runtime underneath) for Parakeet. Rusqlite for history. No Python in the hot path; no Electron. Installer is small; models are downloaded on first run (one-off, ~460 MB for int8).
2. **The audio path** — for microphone input, cpal captures at the device's native rate (typically 44.1 / 48 kHz). For system audio we go direct to WASAPI: `IMMDeviceEnumerator::GetDefaultAudioEndpoint(eRender)` + `AUDCLNT_STREAMFLAGS_LOOPBACK`, event-driven. Either path downmixes to mono f32, resamples to 16 kHz, applies optional silence trim / normalise / noise suppression, then quantises to int16 for the ASR.
3. **Transcription** — on-device Parakeet TDT 0.6B v2, greedy decode by default. Real-world latency on a modern laptop: well under a second after you stop speaking (faster on desktop / GPU).
4. **Post-processing** — deterministic rules run always (filler removal, capitalisation, auto-punctuation, auto-space, trim trailing space). Optional LLM rewrite on Pro via local llama.cpp for tone / style / format changes. A deterministic fallback ships in builds compiled without the llama feature so the app still runs end-to-end.
5. **Delivery** — paste-via-clipboard (restores the user's clipboard ~1.5 s after) by default. If `SendInput(Ctrl+V)` returns 0 (UIPI / elevated target / RDP), we transparently fall back to per-character Unicode keystroke injection so the dictation still lands. Optional shift-to-auto-send fires `Enter` after the paste for chat-box dictation.
6. **What we send over the network** — a single line: _"Model downloads from Hugging Face on first run. That's it. No audio, no text, no telemetry, no phone-home."_

### 5.4 Privacy (`/privacy`)

Not the legal page. This is the marketing/essay page that explains WHY local matters. Legal copy lives at `/legal/privacy`.

Outline:

- **The only dictation app that can't leak your voice.** (H1)
- Threat model: cloud dictation vs Utterbar. A table.
- What Utterbar stores on disk (audio optional, text yes, all in your `%APPDATA%\Utterbar\` folder, all in SQLite, all deletable).
- What Utterbar sends to the network (model downloads only, hash-verified, from a list of allowed hosts).
- Why there's no telemetry (and when/if we add it, it'll be opt-in and you'll see every payload).
- Link to the legal privacy policy + link to the GitHub repo for anyone who wants to audit.

### 5.5 Pricing (`/pricing`)

Three tiers. All CTAs route to waitlist for now.

**Free**
- `£0` / forever
- For trying it out and personal use
- Includes:
  - Parakeet TDT 0.6B v2 (int8, ~460 MB) — the CPU-friendly default, English-only
  - Push-to-talk, toggle, hold-to-talk across seven hotkey slots
  - Default modes: Voice-to-text, Message, Email, Note, Meeting, Super (deterministic post-processing only on Free)
  - Microphone, system-audio, and mixed capture
  - Vocabulary up to 50 words
  - Text replacements up to 25
  - History (last 100 transcriptions)
  - Media auto-pause
  - Full privacy controls

**Pro** — headline tier, visually highlighted
- `£8` / month or `£72` / year (save two months)
- For daily dictators
- Everything in Free, plus:
  - Parakeet TDT 0.6B v2 (fp16, ~1.16 GB, GPU-only — DirectML or CUDA) + any future model additions
  - Unlimited custom modes
  - LLM post-processing via local llama.cpp — tone rewrites, reformatting, summarisation, using optional active-app / selection / clipboard context
  - Unlimited vocabulary + replacements
  - Full history with audio retention (1–90 days) + reprocess with any mode
  - File transcription (wav / mp3 / aac / mp4 / mkv / ogg / flac → txt / srt / vtt)
  - Priority support

**Team**
- `£6` / seat / month, min 3 seats
- For agencies, law firms, medical practices, anyone who can't use cloud dictation
- Everything in Pro, plus:
  - Shared vocabulary and modes
  - SSO (Microsoft 365, Google Workspace)
  - Group billing
  - Centralised privacy policy for admins
  - Named contact for support

Below the tiers:

- FAQ: "Why a paid tier at all?" _Because a real product needs a real engineer paying the bills. Free is free forever, not a trial._
- FAQ: "Can I self-host?" _The app is already self-hosted — it runs on your machine. No server to host._
- FAQ: "Discounts?" _Students and open-source maintainers: email founder@utterbar.app._

### 5.6 Docs (`/docs`)

Sidebar + article layout, MDX-rendered. Ship these stubs with placeholder content ("Coming soon — real docs land with 1.0.") so the information architecture is in place:

- Getting started
- Modes
- Vocabulary & replacements
- Shortcuts
- Privacy controls
- Troubleshooting
- FAQ (duplicated of home FAQ, more exhaustive)

### 5.7 Roadmap (`/roadmap`)

Three-column kanban-style layout. Read-only. Pull items from a static MDX / YAML file for easy editing.

**Now** (in progress for 1.0):
- Signed release pipeline (MSI + NSIS binaries)
- Auto-update UI (setting is wired; installer flow to build)
- File transcription drag-drop polish (decoder + ASR path are already working; the UX round-trip is what's left)
- Onboarding polish + first-run model-download UX

**Shipped** (already in the app — do not list as a "promise"):
- LLM post-processor (local llama.cpp feature-gated; deterministic fallback always available)
- Cycle-mode + open-settings + open-history global hotkeys
- WASAPI loopback capture (system audio / mixed)
- Adaptive noise suppression toggle
- Keystroke fallback for UIPI-blocked apps
- Shift-to-auto-send for chat boxes
- Retain-audio with day-limit slider
- Reduce-motion accessibility
- Synthesised sound cues + toast notifications
- Theme + accent pickers, bar visibility / size / outline

**Next** (post-1.0):
- macOS build
- File transcription batch mode
- Custom model imports (drop a sherpa-onnx tarball in, get a new model)
- Multilingual Parakeet (the 0.6B v2 in the catalogue is English-only)
- Team plan + SSO

**Later** (ideas, no commitment):
- Linux build
- Bilingual dictation modes (switch language mid-sentence)
- Mobile remote (press-to-talk from your phone)
- VS Code extension for true in-IDE cursor injection

### 5.8 Legal pages

`/legal/privacy`, `/legal/terms`, `/legal/eula` — plain article pages, table of contents in sidebar, rendered from `.mdx` files checked into the repo. Use template skeletons from Iubenda / Termly as a starting point but rewrite to the Utterbar tone. Jay will supply the real text before launch.

### 5.9 Press (`/press`)

- One-liner description (copy verbatim from hero sub)
- Download ZIP: logos (SVG + PNG), 5 screenshots, 1 GIF of the pill in action
- Founder bio (3 sentences on Jay)
- Press contact: `press@utterbar.app`

### 5.10 404

Big cyan pill with a waveform drawing a straight line. Copy: _"Silence detected. That page isn't here."_ Link home.

---

## 6. Waitlist

Single most important conversion on the site.

- Blade partial lives at `resources/views/partials/waitlist.blade.php`. Inline on home hero, home final CTA, pricing CTA-all-tiers, and a standalone `/waitlist` page.
- Fields: email (required), optional "what are you going to use Utterbar for" single-line text.
- Backend: Laravel `POST /waitlist` route → `WaitlistController@store` → writes to the `waitlist` MySQL table (`id, email UNIQUE, use_case NULLABLE, ip, user_agent, created_at`). Rate-limit via `throttle:10,1` to stop scrapers.
- On success: set a `waitlisted` cookie so returning visitors see "You're on the list" instead of the form. Turbo / Livewire / plain fetch all fine — whichever the team prefers; Livewire is idiomatic for Laravel and avoids a JSON layer.
- Confirmation email: Mailable class `App\Mail\WaitlistConfirmation` sent via Laravel's `mail` facade. SMTP credentials from Cloudways (or Resend/Postmark add-on — if Resend, use the Laravel Resend driver package, not a REST wrapper).
- Migration: `database/migrations/*_create_waitlist_table.php`. One email per row; `email` has a UNIQUE index so duplicates are a database constraint, not application logic.

Success copy: _"You're in. We'll email once there's a build worth installing."_

Failure (already subscribed): _"You're already on the list — nothing to do."_

---

## 7. FAQ content (verbatim copy)

1. **Does Utterbar send my voice to the cloud?**  
   No. Audio is captured, transcribed, and discarded on your PC. The only network calls Utterbar makes are to download ASR models from Hugging Face on first run. That's visible in the app's logs.

2. **Which Windows versions are supported?**  
   Windows 10 22H2 and Windows 11. We require WebView2 (pre-installed on 11, auto-bootstrapped on 10).

3. **Do I need a GPU?**  
   No. The default model is a Parakeet TDT 0.6B int8 build that runs on CPU comfortably. If you have a GPU, a larger fp16 model is available on Pro — DirectML lights up any DX12 card (NVIDIA / AMD / Intel), CUDA is there for NVIDIA-only setups. On a laptop with an RTX 4060, transcription is effectively instant. On a 4-core CPU from 2019, expect ~0.5× real-time.

4. **How does it compare to Superwhisper on Mac?**  
   Same category, same conviction about local-only ASR, but Windows-native from the ground up. We don't shim through Python, we use sherpa-onnx (ONNX Runtime) directly, and the installer is smaller.

5. **What about Whisper?**  
   We ship with Parakeet TDT because it's faster and more accurate on English than Whisper small/medium. Whisper support is on the roadmap for multilingual modes.

5a. **What languages are supported?**  
    Today, English only. The Parakeet TDT 0.6B v2 models in the catalogue are English-only exports. Multilingual (via Parakeet multi or Whisper) is on the roadmap — we'd rather ship one language well than five badly.

6. **Can I use this in Teams / Zoom / Discord?**  
   Yes, as long as you're typing into the chat box. Utterbar does not (and will not) inject your voice into a call — it transcribes your voice and injects the text. For the avoidance of doubt: your colleagues won't hear anything Utterbar does.

7. **Does it work offline?**  
   Completely, once the model is downloaded. The app doesn't auth against anything.

8. **Will there be a macOS version?**  
   Yes, after Windows 1.0 ships. Superwhisper already serves the Mac market well — we want Windows to be first-class first.

9. **What happens to my data when I uninstall?**  
   `%APPDATA%\Utterbar\` is left in place so you can re-install without losing history. Delete it manually if you want a clean wipe.

10. **Can I audit the code?**  
    The Rust core is open for inspection by researchers and enterprise buyers under NDA. A public open-core repo is on the roadmap.

---

## 8. Tech stack — see MONOREPO.md

The earlier body of this section specified Laravel 11 + Livewire 3 + Plausible + `utterbar.app` + Inter fonts + a waitlist-form folder tree. **None of it matches what shipped.** The website is Laravel 12 + Blade + Alpine (no Livewire) + Tailwind v4, domain is `utterbar.com`, fonts are Instrument Sans + Geist Mono, there is no waitlist (the site sells the app directly via Stripe Checkout), and the folder tree has drifted.

The authoritative description of what actually runs lives in [../MONOREPO.md §3](../MONOREPO.md). Do not follow anything that used to live in this section.

---

## 9. SEO / metadata

- **Default `<title>` template:** `%s — Utterbar` with home overriding to `Utterbar — Dictation that stays on your PC`.
- **Description (default):** _Utterbar is a Windows-native, local-only dictation app. Press a hotkey, speak, type anywhere — offline, private, fast._
- **OG images:** one per route, 1200×630, cyan-on-slate with the page H1 and the Utterbar wordmark bottom-right. Generate via `spatie/browsershot` against a dedicated `/og/{route}` Blade template, or pre-render and drop into `public/og/`. Skip on-the-fly cloud services — the file-based approach caches cheaply through Cloudways Varnish.
- **Twitter card:** `summary_large_image`.
- **Sitemap:** generated by `spatie/laravel-sitemap` via a scheduled artisan command; served from `public/sitemap.xml`.
- **Robots:** allow everything except `/api/*`.
- **Structured data:** `SoftwareApplication` schema on home + pricing.
- **Canonical URLs** on every page.

Target keywords, in order:

1. `windows dictation app`
2. `local dictation windows`
3. `offline speech to text windows`
4. `superwhisper for windows`
5. `parakeet tdt dictation`
6. `private dictation app`

Don't keyword-stuff. Write like a human.

---

## 10. Accessibility

Non-negotiable:

- Colour contrast AA minimum for body text (the brand cyan `#22D3EE` on `#0B0F14` is 8.4:1 — fine). On hover/accent backgrounds ensure foreground text is `#0B0F14`, not white.
- All interactive elements focus-visible with a 2px cyan ring (`outline: 2px solid var(--accent); outline-offset: 2px;`).
- `prefers-reduced-motion`: kill the hero loop, replace with a still frame.
- `prefers-color-scheme`: we're dark-only; don't invert on light mode, but don't break either — just stay dark.
- Every video has captions (English SRT) and `aria-label`. Autoplay muted + `playsinline` everywhere.
- Forms: labels always visible, not placeholder-only. Errors announced via `aria-live="polite"`.
- Keyboard navigation works for nav, footer, accordions, pricing cards, and the waitlist form. Skip-to-content link at the top.

---

## 11. Performance budget

- LCP < 2.0 s on 4G / mid-tier mobile.
- CLS < 0.05.
- JS bundle (home route) < 120 kB gzipped.
- Hero video < 2 MB, H.264 + AV1 fallback, `poster` attribute so the first paint is immediate.
- Fonts: preload Inter Variable (`<link rel="preload" as="font" ... crossorigin>`), defer JetBrains Mono (used in the footer / kbd only).
- Images: AVIF + WebP fallbacks. Use `<picture>` with explicit `width`/`height` to avoid CLS; pre-convert assets in `public/` at build time (tiny shell script or artisan command is fine).
- No third-party scripts on the critical path. Plausible loads async.
- Cache warm routes at the Varnish layer on Cloudways. The home page and `/pricing` should cache for 10 minutes at the edge — `/waitlist` and the docs pages are uncached.

---

## 12. Legal / compliance

- Cookie banner: only if we use non-essential cookies. With Plausible (cookieless) and no ads, we can skip the banner entirely. If the waitlist form stores an email address, add a GDPR-style consent tick ("I'm happy to receive one confirmation email and occasional updates") + link to `/legal/privacy`.
- Data retention: waitlist emails kept until launch, then migrated to the paying-customer DB or deleted on request.
- Right-to-delete: `support@utterbar.app` handles manually for now. A real DSAR flow lands with the billing system.
- Accessibility statement on `/legal` landing.

---

## 13. When you're done

Ship the whole site before any paywall, download, or checkout logic lands. The deliverables for "done-v1":

1. All routes above rendered, mobile and desktop, dark-only.
2. Waitlist live end-to-end (Livewire form → MySQL insert → queued confirmation email).
3. All copy in this file wired in — no `Lorem ipsum` anywhere.
4. Legal pages present even if placeholder text ("Full policy published with 1.0 release").
5. OG images rendering for every route.
6. Sitemap + robots.txt generated and served from `public/`.
7. Lighthouse ≥ 95 on Performance / Accessibility / Best Practices / SEO on `/` and `/pricing`.
8. Deployed to Cloudways on `utterbar.app` with the apex redirect and SSL sorted. Redis and queue worker healthy. `.env.production` values set through the Cloudways panel, never committed.

When the build is green and deployed, ping Jay with the URL and a short report of what's placeholder vs real.

---

## 14. Things NOT to do

- Do not copy any Superwhisper marketing copy. Reference them by name only where listed here.
- Do not add a chatbot widget.
- Do not add a newsletter widget other than the waitlist.
- Do not use stock photos of people wearing headsets. Illustrations or product shots only.
- Do not reference pricing in GBP anywhere other than the pricing page (we'll add USD/EUR later — keep it single-currency for v1).
- Do not publish before Jay has reviewed every page. Deploy to a staging URL (`staging.utterbar.app`) first.

---

## 15. Questions? Defaults.

If something's not covered here, use your judgement and err toward: **less copy, more white space, more cyan, more British English, more local-only emphasis**. When in doubt, borrow the pattern from Linear's marketing site (tone, density, restraint), not from a typical SaaS landing page.

Good luck. Build it confidently — this is the site that sells a product Jay's been working on for months.

---

## 16. Canonical app facts — now lives in config + code, not this brief

This section used to describe itself as the "single source of truth" for app version, supported platforms, installer type, bundle identifier, and canonical domain. It was written against v0.3.5 and has not been updated since. Each of those facts now has a better home:

- **Current version** lives in `Utterbar Windows App/src-tauri/Cargo.toml`, `tauri.conf.json`, and `package.json` (kept in lockstep by the release process). The server reflects what was actually published via the admin-release API (`releases` table) — [../MONOREPO.md §5](../MONOREPO.md) has the pipeline.
- **Platform, installer type, bundle ID** are in `tauri.conf.json`.
- **Canonical domain** is `utterbar.com`. Earlier drafts used `utterbar.app` as a working name; `.app` is not owned and should not be referenced anywhere.

The modes / built-in models / privacy defaults lists that used to live below were useful once but drifted faster than anyone updated them. For current shipped modes see the seed in `Utterbar Windows App/src-tauri/src/services/modes.rs`. For current shipped models see the catalogue at `Utterbar Windows App/src-tauri/src/services/models.rs`. The authoritative copy of "what the app actually does right now" is the app itself and the per-version release notes at `Utterbar Windows App/releases/`.

---

## 17. What's NOT yet in the app — see TODO.md

The list that used to live here was accurate for v0.3 (signed installer absent, auto-update not wired, file-transcription drag-drop incomplete, Team tier unbuilt, multilingual English-only, macOS absent, telemetry absent). Most of those items have moved: file transcription, auto-update, and the licence-gated Pro tier all ship in v0.6.x. Some are still accurate (macOS, multilingual dictation, telemetry opt-in).

Rather than maintain two truth sources that drift apart, the current list of what is and isn't shipped lives in [../TODO.md](../TODO.md) and the per-version release notes at [../Utterbar Windows App/releases/](../Utterbar%20Windows%20App/releases/). If you're writing marketing copy and need to know whether to claim a feature, check those first.

---

## 18. Change log for this brief

Track brief updates here so the next contributor can diff intent against the current state.

- **2026-04-22** — Rewritten feature lists and pricing to reflect the v0.3.5 app: WASAPI loopback capture, keystroke fallback, shift-to-auto-send, retain-audio with day slider, reduce-motion, sound cues, toast notifications, bar visibility/size/outline, themes + accents, GPU selector. Replaced fictional model names ("TDT small 120 MB" / "TDT large 1 GB") with the real catalogue (`sherpa-parakeet-tdt-0.6b-v2-int8` ~460 MB, `-fp16` ~1.16 GB). Fixed modes list (Voice-to-text / Message / Email / Note / Meeting / Super — not "Slack/Chat" or "Code comments"). Moved shipped roadmap items to a new "Shipped" list. Added §16 canonical facts table and §17 "what's not yet in the app" so the site stops over-promising. Flagged `utterbar.app` vs `utterbar.com` domain ambiguity for Jay to resolve.
