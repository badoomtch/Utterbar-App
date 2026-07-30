# Utterbar — Website Brief

> **Scope (2026-04-26):** this file is authoritative for **product positioning, brand voice, and marketing copy tone** only — what Utterbar *is* and how to talk about it. It is NOT authoritative for shipping status, tech stack, route inventory, concrete prices, current visual tokens, or anything else that drifts.
>
> Look elsewhere for:
> - Architecture, stack, current shipping status → [../MONOREPO.md](../MONOREPO.md)
> - Live state and open work → [../TODO.md](../TODO.md)
> - Visual language and tokens → [./DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md)
> - Repo-wide working rules → [../CLAUDE.md](../CLAUDE.md)
> - Release ritual → [../RELEASE_RUNBOOK.md](../RELEASE_RUNBOOK.md)
>
> The domain is `utterbar.com`, not `utterbar.app`. **As of 2026-07-25 Utterbar is free and open source (MIT), with every feature included — there is no price, no Pro tier, no licence, no account.** The paid/licensing machinery was removed in the open-sourcing pass. The public CTA is "Download Utterbar". The `LICENCE_*` docs describe a system that no longer exists; treat them as history. If anything below contradicts these facts, these facts win and this file should be edited.

---

## 1. What Utterbar is

Utterbar is a premium dictation utility for Windows. It sits as a small floating pill on the user's desktop, listens when they hit a global hotkey (default `Ctrl+Space`), transcribes their speech with NVIDIA Parakeet running 100% on-device, and injects the text straight into whichever app is focused — Chrome, VS Code, Notion, Slack, anywhere.

Anchors for every page:

- **Local-only.** No cloud. Audio never leaves the machine. Say it loudly and repeatedly.
- **Windows-first.** macOS users have Superwhisper; Utterbar is built for Windows from day one (no Electron, no WSL, no Python shim in the hot path — Tauri + Rust).
- **Talks to your AI assistant.** Claude / ChatGPT / Gemini / Copilot / your own local model — Utterbar types into whichever app has the cursor, so any chat input becomes a voice input. No browser extension, no API key, no integration. This is one of the strongest reasons people install the app; lead with it on home + features.
- **Multilingual.** 30+ languages out of the box. English uses Parakeet (faster + more accurate at the same size); other languages use Whisper. Same offline guarantee in every language.
- **Fast.** Parakeet TDT (0.6B v3) runs on CPU or GPU; real sentences finish transcribing in well under a second on a modern laptop. DirectML lights up any DX12 GPU (NVIDIA / AMD / Intel); CUDA path for NVIDIA-only users.
- **Works with every app.** Anything that accepts keystrokes accepts Utterbar output. UIPI-locked / elevated targets fall back to per-character keystroke injection automatically.
- **Modes.** Different post-processing per context (Voice-to-text, Message, Meeting are the shipping built-ins as of writing — check the app's seed file before listing modes by name in copy).
- **Capture anything that plays on the PC.** Mic, system-audio (WASAPI loopback, no virtual cable needed), or both summed.
- **Vocabulary.** Teach it names, acronyms, jargon. Those words stop getting mis-transcribed.
- **Privacy switches** for clipboard / selection / active-app context, off by default for the paranoid.

Elevator pitch (use verbatim on hero):

> Dictation that stays on your PC. Utterbar transcribes your voice into any Windows app, instantly, offline, and without sending a single byte to the cloud.

---

## 2. Brand

### 2.1 Name

`Utterbar` — one word, capital U, lowercase rest. Never `UtterBar`, never `utter bar`. The utterance bar. The thing you utter into.

The floating pill in the app is called "the Utterbar" (definite article) in docs and help content. On the website, lean on "Utterbar" as a proper noun.

### 2.2 Visual identity — see DESIGN_SYSTEM.md

Exact hex tokens, gradients, theme rules, and type scale all live in [./DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md) and the live `tokens.css` files in the app and website. Do NOT hardcode values into copy — pull them from there.

What's still authoritative at the identity level (not implementation):

- Electric cyan identity colour. Not navy, not enterprise-SaaS blue. Tape-deck VU-meter glow.
- Dark by default. Cyan is reserved for action, focus, and live-listening cues — never a body-copy tint.
- Three canonical SVG logo sources ship with the app (`src-tauri/icons/mark.svg`, `wave.svg`, `lockup.svg`). Reuse them, do not redraw.
- Clear space around the logo = the height of the LED dot on all four sides.

### 2.3 Iconography

Use `lucide` (matches what the app uses). Stroke 1.5, size 18–20 in body, 24 in features, 32 in the hero quick-stats strip. Never mix icon libraries. Never use emoji as icons — only in the occasional in-context quote.

### 2.4 Tone of voice

Confident, specific, minimal. Three rules:

1. **Lead with the concrete thing.** "Your voice stays on your machine." not "Privacy-first AI-powered workflow".
2. **Name the competition when it sharpens a sentence.** Superwhisper is the Mac equivalent; saying so saves a paragraph.
3. **Short sentences on hero + features.** Headlines max 9 words. Longer paragraphs are fine on docs / FAQ.

Banned words: *synergy, empower, revolutionise, game-changer, AI-powered (as a noun), seamless, unlock, frictionless, supercharge, next-gen, effortless, robust, tailored, intuitive, streamline, elevate, transform the way, designed to help you, everything you need, whether you're, simply, just, easily*. If you reach for one, rewrite.

**No em dashes.** Don't use `—` or `&mdash;` in customer-facing copy. Use a comma, full stop, colon, or semicolon. The em dash is a tell for AI-generated marketing copy, and rewriting around it makes the sentence sharper anyway.

**No fake friendliness.** Avoid "good news", "you're all set", "no worries", "we've got you covered". Plain confidence reads better.

**Sentence rhythm should feel human.** Don't write three perfectly-balanced sentences in a row. Mix lengths. A four-word sentence is fine. So is an eight-word one. So is a longer one with a comma in the middle.

Preferred register: British English spelling (`colour`, `organise`, `customise`, `optimise`, `licence` as noun, `license` as verb). The product ships worldwide; the founder is British; the copy stays British.

---

## 3. FAQ — verbatim copy

Reusable across home + docs/FAQ. Edit in lockstep when the underlying behaviour changes.

1. **Does Utterbar send my voice to the cloud?**
   No. Audio is captured, transcribed, and discarded on your PC. The only network calls Utterbar makes are downloading ASR models on first run and a once-per-launch update check. Neither carries any of your speech, and both are visible in the app's logs.

2. **Which Windows versions are supported?**
   Windows 10 22H2 and Windows 11. We require WebView2 (pre-installed on 11, auto-bootstrapped on 10).

3. **Do I need a GPU?**
   No. The default model is a Parakeet TDT 0.6B int8 build that runs on CPU comfortably. If you have a GPU, larger and more accurate models are available to everyone — DirectML lights up any DX12 card (NVIDIA / AMD / Intel). On a laptop with an RTX 4060 transcription is effectively instant. On a 4-core CPU from 2019, expect ~0.5× real-time.

4. **How does it compare to Superwhisper on Mac?**
   Same category, same conviction about local-only ASR, but Windows-native from the ground up. We don't shim through Python, we use sherpa-onnx (ONNX Runtime) directly, and the installer is smaller.

5. **What about Whisper?**
   We ship Parakeet TDT for English (faster + more accurate at the same size) AND Whisper for everything else. The app picks the right engine per language; users don't have to think about it.

5a. **What languages are supported?**
    Over 30 out of the box: English, French, German, Spanish, Italian, Portuguese, Dutch, Polish, Turkish, Russian, Arabic, Hindi, Japanese, Korean, Mandarin, and more. English uses Parakeet by default; everything else uses Whisper. All processed locally — no exceptions per language. Auto-detect is on by default; pin a language in Settings → Models for a small speed gain.

    *Marketing rule:* claim **"30+ languages"** in copy, not "99". Whisper supports more on paper but quality drops sharply outside the top 30, and a 99-language claim invites disappointment in the long tail. Lead with audience-flavoured language ("Speaks 30+ languages. None of them leaves your PC.") not the count alone.

6. **Can I use this in Teams / Zoom / Discord?**
   Yes, as long as you're typing into the chat box. Utterbar does not (and will not) inject your voice into a call — it transcribes your voice and injects the text. Your colleagues won't hear anything Utterbar does.

7. **Does it work offline?**
   Completely, once the model is downloaded. The app doesn't auth against anything.

8. **Will there be a macOS version?**
   Eventually. Superwhisper already serves the Mac market well — we want Windows to be first-class first.

9. **What happens to my data when I uninstall?**
   `%APPDATA%\Utterbar\` is left in place so you can re-install without losing history. Delete it manually for a clean wipe.

10. **Can I audit the code?**
    The Rust core is open for inspection by researchers and enterprise buyers under NDA. A public open-core repo is on the roadmap.

---

## 4. Things NOT to do

- Do not copy any Superwhisper marketing copy. Reference them by name only where listed here.
- Do not add a chatbot widget.
- Do not add a newsletter widget.
- Do not use stock photos of people wearing headsets. Illustrations or product shots only.
- Do not introduce any price, paid tier, subscription, "Pro" upsell, licence, or account. Utterbar is free and open source; the whole billing/licensing stack was removed. The positioning is "free and open source, every feature included".
- Do not reference `utterbar.app` anywhere — `.app` is not owned. The domain is `utterbar.com`.

---

## 5. Defaults when something isn't covered

When in doubt: **less copy, more white space, more cyan, more British English, more local-only emphasis.** When stuck on tone, borrow the pattern from Linear's marketing site (density, restraint), not from a typical SaaS landing page.

---

## 6. Change log

- **2026-07-25** — Free + open-source pivot. Utterbar is now free, MIT-licensed, every feature on for everyone. Removed the entire paid/licensing stack (Stripe, licence signing + device caps, accounts/magic-link, the subscriber list, volume/Team/Business tiers, tips, the admin web panel). The GitHub Releases-backed auto-updater and the marketing site stay. All "£39.99 / Pro / one-time / volume" copy is rewritten to "free and open source". The prior B2B/pricing entries below are historical.
- **2026-04-27** — B2B pass: dropped the no-Team-tier rule. Volume pricing is live on `/pricing` (Team £29 / Business £19 / Enterprise contact, all one-time, no subscription) and the dedicated `/business` page now positions Utterbar for call centres, legal, healthcare and internal IT. Self-serve checkout still single-seat by design. The local-only / no-cloud / no-account ethos is unchanged and remains the spine of the business pitch.
- **2026-04-26** — Stripped concrete pricing, route inventory, page-by-page copy specs, deployment claims, contact email addresses, accessibility/perf budgets, and the v0.3-era status section. BRIEF.md is now positioning / tone / brand / FAQ only. Anything implementation-level was either already covered elsewhere (MONOREPO, TODO, DESIGN_SYSTEM, RELEASE_RUNBOOK) or had drifted faster than anyone updated this file.
- **2026-04-22** — Earlier rewrite to remove fictional model names and align feature lists with v0.3.5. Superseded by the 2026-04-26 trim.
