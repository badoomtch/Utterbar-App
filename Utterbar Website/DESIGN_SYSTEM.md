# Utterbar — Design System

> Single source of truth for the visual language. The marketing site (Laravel/Blade) already implements this; the desktop app (Tauri/React) should match it one-for-one. Read this end-to-end before changing UI in either codebase.

**Authoritative:** this document supersedes every earlier design note. If something in the code contradicts this, the code is wrong.

---

## 0. Scope and philosophy

Utterbar's visual language is:

- **Warm near-black, blue-slate leaning.** Not cold steel slate, not pure black, not neutral charcoal. A slight blue tint runs through every surface so cyan reads as "on-brand" rather than "accent popped onto greyscale".
- **Typography-led.** Hierarchy happens in weight + scale, not in colour. Body text is readable; display text is loud; mid-weights are silenced.
- **Cyan is the signature, not the atmosphere.** One or two deliberate cyan moments per viewport. Never "cyan everywhere".
- **Flat.** No glossy buttons. No drop-shadow bloom. No faux-3D chrome. The one sanctioned gradient is the 2-stop charcoal pill gradient (`--pill-top → --pill-bot`) shared between the logo mark and the Ghost button — because they're the same object. Atmospheric lighting is allowed (radial washes inside surfaces, long floor shadows under featured objects, soft cyan shadow under the CTA) — glow radiating outward from UI controls as a decorative halo is forbidden.
- **Editorial.** Generous vertical rhythm, varying container widths, hairline rules, mono numbers for chapters. Reads more like a design-annual spread than a SaaS template.
- **Technical without theatre.** No terminal-window chrome, no monospace-for-vibe labels, no status-bar widgets. Technical credibility is conveyed through precision (aligned grids, measured type, honest metadata), not through cosplay.

**Pills are the product.** The app itself is a floating pill on the user's desktop. Every significant UI element (buttons, chips, the active-state rail, the brand mark) echoes that pill's geometry and LED language. That's the north star — the site and app should read as variations on the same physical object.

---

## 1. Palette

All colours are CSS custom properties on `:root`. **Never hardcode hex in components** — reference the variable. If a shade is missing, add it to the palette, don't inline it.

### 1.1 Surfaces (the greyscale ladder)

```css
--bg-0: #0A0E17;   /* page background */
--bg-1: #10151F;   /* main surface — cards, panels, sidebar, footer */
--bg-2: #171C29;   /* raised surface — ghost buttons, keyboard chips, pricing Pro column, active docs item */
```

These three sit on the same blue-slate tonal curve. Never use pure `#000` (kills the cyan). Never introduce a fourth surface without justifying in writing.

### 1.2 Borders (hairlines only, always rgba so they lift on any surface)

```css
--border:        rgba(255, 255, 255, 0.06);   /* default hairline — separators, rare outlines */
--border-strong: rgba(255, 255, 255, 0.10);   /* inputs, keyboard chips, more visible separators */
```

**Never** set a solid-colour border on any card or panel — the card language is "elevated surface via colour", not "outlined box". See §4.

### 1.3 Text

```css
--fg-hi:    #FFFFFF;   /* H1, H2, H3, nav active, emphasised inline text */
--fg:       #E8E8EB;   /* body copy, default UI text, nav inactive, footer links */
--muted:    #8A8A94;   /* secondary text, captions, mono metadata, eyebrows */
--fg-dim:   #5A5A64;   /* extremely quiet metadata (footer mono tokens) */
```

### 1.4 Brand cyan

```css
--accent:   #22D3EE;   /* primary brand cyan — LED indicators, active rails, hover tints */
--accent-2: #67E8F9;   /* hover state / lighter sibling */
--accent-3: #0891B2;   /* deep end of gradients, atmospheric washes */
```

The **in-app pill gradient** is a different, slightly saturated pair: `#00e5ff → #00a3ff`. This pair is used exclusively for:
- The wordmark "bar" letters
- The `<x-hl>` gradient-highlight word component
- The pill mark's internal waveform bars (inside the logo glyph)

Do not substitute `--accent` for `#00e5ff` in those places — the gradient pair is tighter/brighter and looks different. The rule: **on identifiable brand primitives (the mark, the wordmark, gradient keywords), use the `#00e5ff → #00a3ff` pair. Everywhere else, use the `--accent` / `--accent-2` / `--accent-3` variables.**

### 1.4.1 Pill tokens (the logo/button surface)

The charcoal pill gradient used by **both the logo and the CTA button**. Single source of truth — if you change it, it changes everywhere.

```css
--pill-top:     #223042;   /* top of the pill body gradient */
--pill-bot:     #111827;   /* bottom of the pill body gradient */
--pill-top-lit: #2a3a52;   /* hover state top */
--pill-bot-lit: #141e2c;   /* hover state bottom */
```

### 1.5 Semantic (app-only)

```css
--amber: #F59E0B;   /* warnings — in-app only, never on marketing site */
--rose:  #F43F5E;   /* destructive — in-app only */
--moss:  #34D399;   /* success — in-app only */
```

Keep these locked inside the app. The marketing site is monochrome + cyan only.

### 1.6 Cyan usage rules (non-negotiable)

- **At most three distinct cyan elements visible in one viewport.** Count them. If over, cut two.
- **Cyan is never a body text colour.**
- **Cyan is never a card outline.** Use `ub-card-rail` (2px cyan left-bar) if you need a branded edge.
- **Cyan backgrounds are reserved for CTA buttons only** (full cyan fill = "this is the primary action").
- **Gradient cyan text (`#00e5ff → #00a3ff`) is reserved for the wordmark and `<x-hl>` keyword highlights.** Never applied to generic headlines.
- **Cyan glow effects** (drop-shadow / box-shadow cyan at opacity) are only used on: the pill LED, the CTA button LED-dot, active nav underlines, and atmospheric radial washes. Never on cards, never on buttons outside their LED.

---

## 2. Typography

### 2.1 Font families (self-hosted, no CDN)

```css
--font-sans: 'Instrument Sans Variable', ui-sans-serif, system-ui, sans-serif;
--font-mono: 'Geist Mono Variable',       ui-monospace, SFMono-Regular, monospace;
```

Install via npm: `@fontsource-variable/instrument-sans` and `@fontsource-variable/geist-mono`. Import at the top of your stylesheet.

**Never use Google Fonts CDN.** Self-hosting is a privacy and performance requirement.

**App-specific caveat:** in the Tauri app, if self-hosting Instrument Sans bundled with the app is unworkable, fall back to system-ui / Segoe UI. The website's `.ub-wm` (wordmark) class is already pinned to `Instrument Sans Variable, 'Instrument Sans', ui-sans-serif, system-ui, -apple-system, 'Segoe UI', Roboto, Arial, sans-serif` — use the same stack.

### 2.2 Weight discipline

Use only these weights. **Do not use weight 600** — it reads as an uncertain middle-ground and flattens hierarchy.

| Weight | Use for |
|---|---|
| 300 | Lead paragraphs, display sub-headlines, the "No cloud. No nonsense." row |
| 400 | Body text (default) |
| 500 | UI labels, nav links, keyboard chips, eyebrow tags, button labels (ghost) |
| 700 | H3 |
| 800 | H1, H2, display headlines |

### 2.3 Type scale

```
Display     clamp(2.75rem, 6vw, 5rem)       weight 800, tracking -0.045em, line-height 1.05
H1          clamp(2.75rem, 6vw, 5rem)       weight 800, tracking -0.045em, line-height 1.05
H2          clamp(2rem,  4vw, 3.25rem)      weight 800, tracking -0.035em, line-height 1.05
H3          1.5rem                          weight 700, tracking -0.02em,  line-height 1.2
Lead        1.25rem (20px)                  weight 300, line-height 1.55, colour --muted     [.ub-lead]
Body        1.0625rem (17px)                weight 400, line-height 1.65
Small       0.875rem (14px)                 weight 400, line-height 1.5
Eyebrow     0.75rem (12px)                  weight 500, letter-spacing +0.12em, UPPERCASE, --muted [.ub-eyebrow]
Mono        0.8125rem (13px)                Geist Mono, weight 400 or 500 — literal data only
```

**No 18–20px "subheading" tier.** That range is the clutter zone; silence it. Content is either a heading (H1–H3) or body / lead.

### 2.4 Mono discipline

Geist Mono appears only for:
- Version strings in the footer
- Keyboard chip labels (`Ctrl`, `Space`)
- File paths (`%APPDATA%\Utterbar\`)
- Timestamps and hash-style IDs
- Editorial mono numbers (`01`, `02`, …) — features, FAQ, how-it-works numbering

**Never** use mono for eyebrows, decorative tags, section labels, or "technical vibe". If the text isn't literal code/data, it's not mono.

---

## 3. Spacing, grid, layout

### 3.1 Vertical rhythm

| Context | Space |
|---|---|
| Between major sections | **160–240px desktop**, 96px mobile. `py-20 md:py-28` (pb-24 md:pb-32 for long-form) |
| Between H2 and its content | 48–64px (mb-12 md:mb-16) |
| Inside a card | 28–40px padding (1.75–2.5rem) |
| Between paragraph and bullet list | 24px |

### 3.2 Container widths (vary by content type)

| Content | Max width |
|---|---|
| Hero | Container 1200px, internal content 5xl (~64rem / 1024px) |
| Reading / pull quote / founder note | `max-w-3xl` (~48rem / 768px) |
| Feature grid / docs article | `max-w-5xl` (~80rem / 1280px) |
| Pricing comparison block | `max-w-5xl` |
| Closing CTA | `max-w-2xl` |

Varying the max-width per content type is the cheapest escape from "generic template".

### 3.3 Grid philosophy

- 12-col base. Break it deliberately (asymmetric features: 4/8, 7/5, etc.).
- **Never** a fully symmetric 3×3 grid of identical cards. If you have six items, stagger or use 2×3 with a layout break.
- Mobile: always stack single-column.

---

## 4. Surfaces, borders, radii

### 4.1 The surface ladder

- **`--bg-0`** = page background. No card sits directly on this without intention.
- **`--bg-1`** = default card / panel. This is the "elevated" surface by default.
- **`--bg-2`** = raised/featured surface. Reserved for: the Pro pricing half, the active docs sidebar item, ghost buttons, keyboard chips, highlighted-mode indicators in-app.

**Elevate via colour, not via border or shadow.** A card is a card because its background is lighter than the page, not because it has a ring around it.

### 4.2 Radii

```
Panels / cards:        18px     (.ub-card, radius via Tailwind rounded-[18px] or CSS)
Big featured blocks:   24px     (pricing comparison, closing CTA containers — rounded-2xl)
Buttons:               999px    (full pill)
Inputs:                10px
Keyboard chips:        6px
Mode chips (in-app):   999px    (full pill, smaller scale)
Pill mark:             intrinsic (rx=13 in a 26-height SVG — do not change)
```

### 4.3 Borders

- **Default: no border on cards or panels.** Their identity comes from background colour.
- **Hairline rule** (`1px solid var(--border)`) is used only for:
  - Section dividers (a thin horizontal rule between content bands)
  - The footer top border
  - Between pricing columns (optional — prefer tonal shift instead)
- **`--border-strong`** is used on: inputs, keyboard chips, ghost button outlines.

### 4.4 Shadows

**Card shadows: none.** No `box-shadow: 0 4px 12px` anywhere. That's SaaS tropes.

**Permitted shadow patterns:**
- **Floor shadow under the pill hero object** (site only): `0 80px 120px -40px rgba(0,0,0,0.5)`. Suggests weight.
- **Cyan glow on the LED** (inside SVG filters, and on the CTA button dot): multi-layered `0 0 Npx rgba(0, 246, 255, alpha)`. This is the only place cyan glows outward.

### 4.5 The card-rail (signature)

Add the class `.ub-card-rail` alongside `.ub-card` to apply the cyan left-rail with a subtle bloom:

```css
.ub-card-rail {
    position: relative;
    border-left: 2px solid var(--accent);
}
.ub-card-rail::before {
    content: '';
    position: absolute;
    top: 0; bottom: 0;
    left: -2px;
    width: 2px;
    box-shadow: 0 0 20px color-mix(in oklab, var(--accent) 45%, transparent);
    pointer-events: none;
}
.ub-card-rail:hover {
    border-left-color: var(--accent-2);
}
```

**Use for:** featured cards (home feature grid, docs overview tiles, how-it-works principle cards), founder pull quote (inline version — no panel, just the absolute-positioned rail), pricing Pro column edge (via internal glow; not explicit rail).

**The card-rail is a signature motif — use it consistently both on site and in the app for any "elevated / featured / selected / active" state.**

---

## 5. Buttons (two tiers + ghost hairline)

All buttons share the `.ub-btn` base: pill shape (999px radius), 0.8rem × 1.5rem padding, font-size 0.95rem, weight 500, line-height 1, 1px transparent border slot.

### 5.1 CTA — white pill with neon-cyan edge (maximum prominence)

Used for: **Join the waitlist, Download, Purchase, Get launch pricing**. Maximum visual weight. High-contrast white fill against the dark page + a cyan border reads as "the thing to click". The cyan atmospheric shadow beneath suggests the button is casting light.

```css
.ub-btn-cta {
    padding: 0.95rem 1.75rem 0.95rem 2.4rem;
    background: #FFFFFF;
    color: var(--pill-bot);
    font-size: 0.975rem;
    font-weight: 600;
    border: 2px solid var(--accent);
    box-shadow:
        0 0 0 1px color-mix(in oklab, var(--accent) 35%, transparent),
        0 8px 28px -8px color-mix(in oklab, var(--accent) 45%, transparent);
}
.ub-btn-cta::before {
    content: '';
    position: absolute;
    left: 1.05rem;
    top: 50%; transform: translateY(-50%);
    width: 9px; height: 9px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow:
        0 0 10px color-mix(in oklab, var(--accent) 80%, transparent),
        0 0 3px color-mix(in oklab, var(--accent) 90%, #fff);
    pointer-events: none;
}
.ub-btn-cta:hover {
    background: #FFFFFF;
    color: var(--pill-bot);
    border-color: var(--accent-2);
    box-shadow:
        0 0 0 1px color-mix(in oklab, var(--accent) 50%, transparent),
        0 12px 36px -8px color-mix(in oklab, var(--accent) 65%, transparent);
}
.ub-btn-cta:hover::before {
    box-shadow:
        0 0 14px color-mix(in oklab, var(--accent) 90%, transparent),
        0 0 4px color-mix(in oklab, var(--accent) 90%, #fff);
}
```

### 5.2 Ghost / Secondary — the logo pill itself (quieter)

Used for: **See the full pricing, Read the privacy policy, Request source review, secondary navigation actions**. Never the main action on a page. Colours are **identical to the logo mark** — same pill gradient, same white border, same white dot. Reads as a mini Utterbar.

```css
.ub-btn-ghost {
    padding-left: 2.2rem;
    padding-right: 1.6rem;
    background: linear-gradient(180deg, var(--pill-top) 0%, var(--pill-bot) 100%);
    color: #FFFFFF;
    font-weight: 500;
    border: 2px solid #FFFFFF;
}
.ub-btn-ghost::before {
    content: '';
    position: absolute;
    left: 0.95rem;
    top: 50%; transform: translateY(-50%);
    width: 8px; height: 8px;
    border-radius: 50%;
    background: #FFFFFF;
    pointer-events: none;
}
.ub-btn-ghost:hover {
    background: linear-gradient(180deg, var(--pill-top-lit) 0%, var(--pill-bot-lit) 100%);
    color: #FFFFFF;
    border-color: #FFFFFF;
}
```

### 5.3 The design story

The two buttons are a **paired light/dark hierarchy** against the dark page:

- **CTA = white pill, neon cyan edge.** High-contrast bright surface + cyan halo. The most prominent, most obvious "click here". Dark charcoal text (`--pill-bot`) reads cleanly on the white. The cyan atmospheric shadow suggests the button is emitting light.
- **Ghost = the logo pill itself.** Dark charcoal body, white 2px border, white dot. Same ingredients as the brand mark — the button literally looks like a miniature Utterbar. Reads as "this is still Utterbar, but quieter".

**Use CTA for primary actions. Use Ghost for secondary actions. No third middle tier.**

`.ub-btn-primary` exists in the codebase as a neutral hairline fallback, but should almost never be used in practice. If you find yourself reaching for it, use Ghost instead.

### 5.4 Sizes

Default is one size. If you need a smaller version (e.g. inline action button in-app), reduce padding proportionally but preserve the LED-dot ratio.

---

## 6. Inputs, chips, keyboard keys

### 6.1 Text inputs

```css
input[type="text"], input[type="email"] {
    background: var(--bg-0);          /* sunken — one step BELOW page */
    color: var(--fg);
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    padding: 0.75rem 1rem;
    font-family: var(--font-sans);
    font-size: 1rem;
    transition: border-color 180ms ease;
}
input:focus {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-color: color-mix(in oklab, var(--accent) 40%, var(--border-strong));
}
```

### 6.2 Keyboard chips (`<kbd>` pills)

```css
.ub-kbd {
    display: inline-flex;
    align-items: center;
    padding: 0.15rem 0.5rem;
    min-width: 1.75rem;
    border-radius: 6px;
    background: var(--bg-2);
    border: 1px solid var(--border-strong);
    border-bottom-width: 2px;        /* tactile "key" feel */
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: 0.8rem;
    font-weight: 500;
}
```

The thicker bottom border gives a physical key feeling. Used identically on site and in-app.

### 6.3 Mode chips / tags (app-specific)

- Inactive: `--bg-2` bg, `--fg` text, full pill (999px radius), no border.
- Active: `--accent` bg, dark text, full pill — matches the CTA button's "lit" energy.
- With live indicator: prepend the cyan LED dot (7–9px, glowing) — identical to the CTA button's cyan dot.

---

## 7. Navigation & links

### 7.1 Nav link (site + app topbar)

```css
.ub-nav-link {
    position: relative;
    color: var(--fg);                /* brightens to fg-hi on hover/active */
    font-size: 0.975rem;
    font-weight: 500;
    letter-spacing: -0.01em;
    padding: 0.5rem 0;
    transition: color 200ms ease;
    text-decoration: none;
}
.ub-nav-link::before {  /* hover sweep — cyan gradient underline fills in from left */
    content: '';
    position: absolute;
    left: 0; right: 0;
    bottom: -2px;
    height: 2px;
    background: linear-gradient(90deg,
        transparent 0%,
        color-mix(in oklab, var(--accent) 70%, transparent) 30%,
        var(--accent) 50%,
        color-mix(in oklab, var(--accent) 70%, transparent) 70%,
        transparent 100%);
    transform: scaleX(0);
    transform-origin: left center;
    transition: transform 360ms cubic-bezier(0.2, 0.8, 0.2, 1), opacity 200ms ease;
    opacity: 0;
    pointer-events: none;
}
.ub-nav-link:hover { color: var(--fg-hi); }
.ub-nav-link:hover::before { transform: scaleX(1); opacity: 1; }
.ub-nav-link.is-active { color: var(--fg-hi); }
.ub-nav-link.is-active::after {  /* active = short centred underline */
    content: '';
    position: absolute;
    left: 50%; bottom: -2px;
    transform: translateX(-50%);
    width: 22px; height: 2px;
    background: var(--accent);
    border-radius: 2px;
    box-shadow: 0 0 10px color-mix(in oklab, var(--accent) 60%, transparent);
}
.ub-nav-link.is-active:hover::before { opacity: 0; }  /* marker takes over */
```

### 7.2 Footer link (same cyan sweep, shorter)

```css
.ub-footer-link {
    position: relative;
    color: var(--fg);
    font-size: 0.9rem;
    text-decoration: none;
    padding: 0.15rem 0;
    transition: color 200ms ease;
}
.ub-footer-link::after {
    content: '';
    position: absolute;
    left: 0; bottom: 0;
    width: 100%; height: 1px;
    background: linear-gradient(90deg, #00e5ff 0%, #00a3ff 100%);
    transform: scaleX(0);
    transform-origin: left center;
    transition: transform 320ms cubic-bezier(0.2, 0.8, 0.2, 1);
}
.ub-footer-link:hover { color: var(--fg-hi); }
.ub-footer-link:hover::after { transform: scaleX(1); }
```

### 7.3 Docs sidebar link (app equivalent: settings sidebar, mode list)

```css
.ub-docs-link {
    position: relative;
    display: block;
    padding: 0.55rem 0 0.55rem 1rem;
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--fg);
    text-decoration: none;
    border-left: 2px solid transparent;
    transition: color 180ms ease, border-left-color 180ms ease, background 180ms ease;
}
.ub-docs-link:hover {
    color: var(--fg-hi);
    border-left-color: rgba(255, 255, 255, 0.18);
}
.ub-docs-link.is-active {
    color: var(--fg-hi);
    border-left-color: var(--accent);
    background: linear-gradient(90deg, color-mix(in oklab, var(--accent) 8%, transparent), transparent 80%);
}
.ub-docs-link.is-active::before {
    content: '';
    position: absolute;
    top: 0; bottom: 0; left: -2px;
    width: 2px;
    box-shadow: 0 0 12px color-mix(in oklab, var(--accent) 50%, transparent);
}
```

**This is the active-state motif for the whole app.** Every "selected" item in the app — current mode in the sidebar, selected history entry, current settings tab — uses **2px cyan left-rail + `--bg-2` background + white text**. It's the `.ub-card-rail` pattern applied to list items. Consistent across site and app.

---

## 8. Recurring signature motifs (the "Utterbar fingerprints")

These are the patterns that, used repeatedly, make something feel "like Utterbar" instead of generic dark UI. Apply them consistently in the app.

### 8.1 The LED dot

A 5–7px cyan circle with a double-layered cyan glow. Meanings:
- **On a button (Ghost/Secondary)**: "this action is live/ready".
- **As a status indicator**: "connected" / "recording" / "active".
- **As typographic punctuation**: rarely, as a bullet before "live" labels.

```css
.ub-led {
    width: 7px; height: 7px; border-radius: 50%;
    background: var(--accent);
    box-shadow:
        0 0 10px var(--accent),
        0 0 3px color-mix(in oklab, var(--accent) 90%, #fff);
}
```

**In the app:** use this same primitive for the mic-active indicator, the "recording now" state on the pill, the live transcription caret, and any "on/off" status on settings pages.

### 8.2 The 2px cyan left-rail

Already documented in §4.5 (cards) and §7.3 (active list items). **This is the single most important brand motif.** Used anywhere content is "featured", "selected", "active", or "important". Always 2px, always `--accent`, always with the soft bloom pseudo-element.

### 8.3 The wordmark (Utter + bar)

```html
<span class="ub-wm"><span>Utter</span><span class="ub-wm-bar">bar</span></span>
```

```css
.ub-wm {
    white-space: nowrap;
    font-family: 'Instrument Sans Variable', 'Instrument Sans', ui-sans-serif, system-ui, -apple-system, 'Segoe UI', Roboto, Arial, sans-serif;
    font-synthesis: none;
}
.ub-wm-bar {
    background: linear-gradient(90deg, #00e5ff 0%, #00a3ff 100%);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    color: transparent;
}
```

Use everywhere the brand name appears as a heading or as text inside another heading. The "Utter" inherits its colour from its parent (usually `--fg-hi`); the "bar" is always the cyan gradient.

### 8.4 Gradient keyword highlight

The `<x-hl>` component. Applies the same cyan gradient (`#00e5ff → #00a3ff`) to a single emphasised word in a heading:

```html
<h1>Your voice <x-hl>never</x-hl> leaves.</h1>
```

**Rule: one `<x-hl>` per heading, and one heading-with-hl per page.** This is emphasis — it stops being emphasis if used on every heading. Pick the load-bearing word.

### 8.5 Atmospheric cyan wash (radial, subtle)

A radial gradient positioned at a corner, `color-mix(in oklab, var(--accent) 10-18%, transparent)` fading to transparent by 55–70%. Used to "light a room" — suggests the page has a cyan light source somewhere off-camera.

Appears in:
- Hero section top-right (animated breathing — see §10)
- Pricing Pro column top-right corner
- Privacy "Utterbar" column top-left corner

**Use sparingly.** At most one atmospheric wash per section, and most sections should have none. Never on the whole page, never on cards.

```css
/* Example: pricing Pro half */
background:
    radial-gradient(120% 80% at 100% 0%, color-mix(in oklab, var(--accent) 12%, transparent), transparent 60%),
    color-mix(in oklab, var(--bg-2) 80%, var(--bg-1));
```

### 8.6 Neon cyan underline (for one phrase per page)

```css
.ub-neon-underline {
    background-image: linear-gradient(90deg, var(--accent), var(--accent));
    background-size: 100% 2px;
    background-repeat: no-repeat;
    background-position: 0 100%;
    padding-bottom: 0.18em;
    filter: drop-shadow(0 0 8px color-mix(in oklab, var(--accent) 55%, transparent));
}
```

Used once on home — the "Most dictation apps charge rent on your voice." line. Optional signature move for longer-form bodies of text.

### 8.7 Mono numbers for chapters / sequences

`01`, `02`, `03` in Geist Mono, 12px, weight 500, `--muted`, UPPERCASE with +0.08em letter-spacing — used as editorial markers on features sections, FAQ rows, how-it-works steps.

### 8.8 The hero floor shadow (site only)

When rendering the pill as a hero object, use `filter: drop-shadow(0 80px 120px -40px rgba(0,0,0,0.6))` so it looks like it's sitting on a lit floor. Do not use this on smaller instances of the pill.

---

## 9. Motion

### 9.1 Principles

- **Still by default.** If motion doesn't earn its keep, remove it.
- **No scroll-triggered fade-ins** on section content. That pattern is the 2024 AI-template tell.
- **No bouncy spring physics.** Everything eases.
- **Respect `prefers-reduced-motion`.** Set animation durations to `0.001ms` under the media query.

### 9.2 Permitted motion

| Where | What | Timing |
|---|---|---|
| Nav link hover | Cyan underline sweeps in from left | 360ms cubic-bezier(0.2, 0.8, 0.2, 1) |
| Footer link hover | Cyan underline sweeps in from left | 320ms cubic-bezier(0.2, 0.8, 0.2, 1) |
| Button hover | Background/border colour transition | 180–220ms ease |
| Card / link hover | Colour transition only | 180ms ease |
| CTA button LED glow | Bigger/brighter on hover | 220ms ease |
| FAQ accordion | Height transition + chevron rotate | 250ms cubic-bezier(0.2, 0.8, 0.2, 1) |
| Hero atmospheric glow | Slow breathing (scale + translate) | 11s ease-in-out infinite |
| Pill waveform animation (app pill only) | LED pulse + wave bars | 1.4–3.2s ease-in-out infinite |
| Cross-page navigation (site) | View Transitions API | 200ms cubic-bezier(0.2, 0.8, 0.2, 1) |

### 9.3 Reduce-motion block

```css
@media (prefers-reduced-motion: reduce) {
    *, *::before, *::after {
        animation-duration: 0.001ms !important;
        animation-iteration-count: 1 !important;
        transition-duration: 0.001ms !important;
    }
}
```

### 9.4 Page-load smoothing (site-specific)

Two techniques in use — both belong in the app if it ever has a web-facing surface:

```html
<!-- paint-early bg, prevents any white flash -->
<style>html,body{background-color:#0A0E17;color:#E8E8EB;}</style>

<!-- instant.page: pre-fetch links on hover for SPA-like feel -->
<script src="//instant.page/5.2.0" type="module" integrity="sha384-..."></script>
```

```css
/* View Transitions API for cross-document navigation */
@view-transition { navigation: auto; }
::view-transition-old(root),
::view-transition-new(root) {
    animation-duration: 200ms;
    animation-timing-function: cubic-bezier(0.2, 0.8, 0.2, 1);
}
```

---

## 10. Brand primitives

### 10.1 The pill mark (the logo)

**Current production design** (this supersedes any earlier pill-with-top-LED variant found in older `src-tauri/icons/mark.svg`). Normalised SVG viewBox `0 0 120 60`.

**Structure:**
- Dark charcoal **pill body**: rounded rectangle 120×60, `rx=30` (fully pill-rounded), gradient top → bottom = `#223042 → #111827` (the `--pill-top → --pill-bot` tokens).
- **Crisp 2px white stroke** (`#ffffff`) around the pill body. This is the single most recognisable visual feature of the mark — the high-contrast white border reads cleanly against any background (dark, light, photo).
- **White hole-punch circle** on the left: `cx=20, cy=30, r=8`, filled `#ffffff`. Acts as a visual anchor and reads as a "power indicator / speaker hole".
- **Internal waveform**: 7 cyan vertical bars + a central speech-bubble tail, arranged symmetrically at `x = 44, 54, 64, 74 (centre), 84, 94, 104`. Heights step up toward centre: 10, 18, 28, 44 (centre with tail), 28, 18, 10. All coloured with the `#00f6ff → #00a3ff` vertical cyan gradient.
- **Central bar with speech-bubble tail**: `<path d="M 74 10 A 2 2 0 0 1 78 10 V 50 L 74 44 Z"/>` — a rounded-top rectangle that tapers to a tail at the bottom-**right**. This is the "dictation app" signal; the right-pointing tail suggests speech projecting forward.

**Canonical files:**
- `resources/views/components/logo.blade.php` — the Blade component, authoritative rendering for the website.
- `public/favicon.svg` — standalone glyph (same geometry) for browser tabs, social-preview fallback.
- `public/app-icon.svg` — 256×256 squircle version for app icon / press-kit use (see §10.4).

**For the Tauri app:** copy the glyph exactly. Update `src-tauri/icons/mark.svg` to match this new geometry. The old "pill-with-top-LED" mark should be retired — it's the previous iteration.

**Never redraw.** If you need a new size, reuse the SVG and scale. Only acceptable modifications: 5-stop gradient refinement and `color-interpolation="linearRGB"` for banding-free cyan.

### 10.1.1 The four canonical usages

1. **Glyph-only (wordless mark)** — viewBox `0 0 120 60` (with 2px padding so the stroke doesn't clip: `viewBox="-2 -2 124 64"`). Use for favicon, tray icon, social avatars, and anywhere the brand needs to appear without the word.

2. **Horizontal lockup (glyph + wordmark)** — glyph + Instrument Sans "Utterbar" wordmark immediately to the right, separated by ~12–14px gap, centre-aligned vertically. This is the marketing-default lockup. See §10.2 for wordmark spec.

3. **App icon (squircle background)** — 256×256 rounded square (rx=56) with a dark gradient background (`#1e293b → #090e17`), 2px `#334155` stroke, and the glyph scaled 1.6× and centred. Drop-shadow on the glyph for premium depth. See §10.4.

4. **Monochrome inverse** — for light-background press photography: white pill with a dark `#111827` stroke and a dark hole-punch. Optional — only if marketing ever ships print material.

### 10.2 Wordmark

See §8.3. The letters "Utter" are in `--fg-hi` (white) by default; "bar" is always the cyan gradient. Font: Instrument Sans Variable at weight 500–600, tracking -0.035em.

### 10.4 The app icon (256×256 squircle)

For installers, taskbar, press kits, and any "packaged product" context:

```
Outer shape:    256×256 rounded square (rx=56) — iOS/macOS squircle proportion
Outer fill:     linear-gradient(180deg, #1e293b 0%, #090e17 100%)
Outer stroke:   #334155 at 2px
Drop shadow:    feDropShadow dx=0 dy=10 stdDeviation=12, black at 0.55 opacity (on the glyph group)

Inner glyph:    The canonical pill mark (§10.1), scaled 1.6×, centered inside the squircle
                (translate(32, 80), scale(1.6)) → 192×96 glyph on a 256×256 background
```

Canonical file: `public/app-icon.svg`. Used for:
- App installer icon
- Windows taskbar / jumplist
- Social previews (Open Graph, Twitter card fallback)
- Press kit

**For the Tauri app icon pipeline:** rasterise `public/app-icon.svg` to the required sizes (16, 32, 48, 64, 128, 256, 512) via `scripts/generate-icon.py` → `pnpm tauri icon`. The squircle acts as the "container" for Windows taskbar rendering.

### 10.3 `<x-wordmark />` Blade component

```html
<span class="ub-wm"><span>Utter</span><span class="ub-wm-bar">bar</span></span>
```

When porting to React for the app, create an equivalent `<UtterbarWordmark />` component with identical markup and class names.

---

## 11. Web → app mapping (the hand-off)

Every rule above applies to the app. Additional translations for desktop-software-specific contexts:

### 11.1 Densities

The app is denser than the marketing site. Scale typography down:

| Token | Site | App |
|---|---|---|
| Body text | 17px | 13–14px |
| Headings | Display scale | 16–22px |
| Section paddings | 160–240px vertical | 32–64px vertical |
| Card padding | 28–40px | 16–24px |

**Do not change the palette, cyan usage rules, font families, or motion philosophy.** Only scale density.

### 11.2 Window chrome

- **Windows 11:** use Mica/acrylic where the Tauri window frame supports it, but apply the app's own `--bg-0` as a fallback/background-color so it still reads as the Utterbar palette rather than system-default.
- **Always show the app as part of the Utterbar visual family** — if a user alt-tabs between the pill and Settings, the Settings window should feel like "Utterbar, larger" not "generic Windows dialog".

### 11.3 Sidebar / nav

```
Sidebar width:     240px
Sidebar bg:        var(--bg-1)
Right border:      1px solid var(--border)
Section headers:   .ub-eyebrow styling
Mode list items:   .ub-docs-link styling (active = 2px cyan left-rail + bg-2 + white text)
```

### 11.4 Active states in the app

Every "selected" / "active" / "highlighted" element uses the **2px cyan left-rail + lifted-bg (`--bg-2`) + white text** combo. This includes:

- Current mode in the sidebar
- Selected history entry
- Active settings tab (sub-navigation)
- Currently-dictating mode indicator
- Selected vocabulary row

**No exceptions.** The rail is the signature — use it consistently.

### 11.5 Recording state (the pill + app UI)

- **Recording active:** pill LED pulses `#00F6FF` (brightest cyan); if the app window is visible, a matching cyan status indicator appears in the app's topbar.
- **Processing (post-recording):** LED stays cyan, waveform bars fade to `--accent-3` (dim), spinning caret on transcript area.
- **Error:** LED pulses `--rose`; this is the only time the rose colour appears in the pill.

### 11.6 Transcript output

- Transcript text: Instrument Sans, 15px, weight 400, `--fg`, line-height generous.
- New text streaming in: cyan caret (`|`) at end, blinking. Same LED cyan.
- Timestamps / metadata in the transcript panel: Geist Mono, 12px, `--fg-dim`.

### 11.7 Audio waveform

Reuse the pill's geometric waveform bars (`#00e5ff → #00a3ff` gradient) for any live audio-level visualisation. Animate bar heights based on real input amplitude. Do not invent a new waveform style.

### 11.8 Don't repeat-brand inside the app

Don't render the wordmark repeatedly throughout the app. Once in the About dialog. Once (optionally) in the sidebar header. That's it. Inside the app, the **product is the interface** — the user knows what it is.

---

## 12. Do / Don't

### Do

- Use `--accent` variables; never hardcode cyan hex.
- Elevate surfaces via background colour, not border/shadow.
- Use the 2px cyan left-rail for any active/selected/featured state.
- Use the LED dot (solid cyan, glowing) for "live" indicators.
- Use Instrument Sans for everything except literal technical data.
- Use Geist Mono only for literal technical data (versions, paths, keys, hashes, timestamps).
- Use `<x-hl>` (or equivalent) for one key word per major heading.
- Give sections generous vertical breathing room.
- Vary max-widths by content type.
- Animate the pill's waveform when recording; keep everything else still.

### Don't

- Don't use pure black (`#000`).
- Don't use monospace for decoration / "vibe".
- Don't put 1px outlines on cards.
- Don't use glossy / faux-3D / "lit gem" buttons, gradient backgrounds on whole sections, or multiple gradient-text moments per page. The only permitted gradient-filled button is the Ghost (the shared charcoal pill gradient with the logo).
- Don't use cyan as a body-text colour, an icon colour, or a hover tint on arbitrary UI.
- Don't use faux-3D gloss, shine, or LED chrome on buttons — they're flat pills.
- Don't scroll-trigger fade-ins on section content.
- Don't use bouncy spring physics anywhere.
- Don't repeat the wordmark in the app UI.
- Don't introduce a new surface colour without adding it to the palette.
- Don't reintroduce retired motifs: the terminal-window CTA, the monospace status-strip pill, glossy / "LED chrome" buttons with gradient inner highlights, icon-card feature grids with heavy paragraph bodies.
- **Don't put an eyebrow label directly above a page H1.** The H1 stands on its own. Eyebrows are for mid-page section labels (inside `<x-section>` above an H2) and for card/component sub-labels ("Still here?", "Before you write", etc.) — never as a breadcrumb or category tag stacked on top of the page title.

---

## 13. Appendix A: complete CSS custom properties

Drop this into the app's global stylesheet (scoped under `:root`):

```css
:root {
    color-scheme: dark;

    /* Surfaces */
    --bg-0: #0A0E17;
    --bg-1: #10151F;
    --bg-2: #171C29;

    /* Borders */
    --border:        rgba(255, 255, 255, 0.06);
    --border-strong: rgba(255, 255, 255, 0.10);

    /* Text */
    --fg-hi:  #FFFFFF;
    --fg:     #E8E8EB;
    --muted:  #8A8A94;
    --fg-dim: #5A5A64;

    /* Brand cyan (variable family) */
    --accent:   #22D3EE;
    --accent-2: #67E8F9;
    --accent-3: #0891B2;

    /* Logo + CTA pill (single source of truth — buttons + logo share these) */
    --pill-top:     #223042;
    --pill-bot:     #111827;
    --pill-top-lit: #2a3a52;
    --pill-bot-lit: #141e2c;

    /* Semantic (app only) */
    --amber: #F59E0B;
    --rose:  #F43F5E;
    --moss:  #34D399;

    /* Cyan gradient (for wordmark, pill mark, <x-hl>) */
    --gradient-cyan:
        linear-gradient(135deg, var(--accent-3) 0%, var(--accent) 55%, var(--accent-2) 100%);
    --gradient-wordmark:
        linear-gradient(90deg, #00e5ff 0%, #00a3ff 100%);

    /* Fonts */
    --font-sans:
        'Instrument Sans Variable', 'Instrument Sans', ui-sans-serif,
        system-ui, -apple-system, 'Segoe UI', Roboto, Arial, sans-serif;
    --font-mono:
        'Geist Mono Variable', ui-monospace, SFMono-Regular, Menlo, monospace;
}
```

---

## 14. Appendix B: component reference

Drop-in stylesheet blocks for the app's design system. All patterns are production-tested on the marketing site.

### .ub-card + .ub-card-rail

```css
.ub-card {
    background: var(--bg-1);
    border-radius: 18px;
    padding: 1.75rem;
    transition: background 200ms ease;
}
.ub-card:hover { background: var(--bg-2); }

.ub-card-rail {
    position: relative;
    border-left: 2px solid var(--accent);
}
.ub-card-rail::before {
    content: '';
    position: absolute;
    top: 0; bottom: 0;
    left: -2px;
    width: 2px;
    box-shadow: 0 0 20px color-mix(in oklab, var(--accent) 45%, transparent);
    pointer-events: none;
}
.ub-card-rail:hover { border-left-color: var(--accent-2); }
```

### .ub-eyebrow + .ub-lead

```css
.ub-eyebrow {
    color: var(--muted);
    font-family: var(--font-sans);
    font-size: 0.75rem;
    font-weight: 500;
    letter-spacing: 0.12em;
    text-transform: uppercase;
}
.ub-lead {
    font-size: 1.25rem;
    line-height: 1.55;
    font-weight: 300;
    color: var(--muted);
}
```

### .ub-kbd (keyboard chip)

```css
.ub-kbd {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.15rem 0.5rem;
    min-width: 1.75rem;
    border-radius: 6px;
    background: var(--bg-2);
    border: 1px solid var(--border-strong);
    border-bottom-width: 2px;
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: 0.8rem;
    font-weight: 500;
    line-height: 1.4;
}
```

### Buttons

(See §5 for the full blocks — copy those verbatim.)

### Active list item pattern

Any "selected" item in a list (sidebar mode, history row, settings tab):

```css
.item-active {
    background: var(--bg-2);
    color: var(--fg-hi);
    border-left: 2px solid var(--accent);
    position: relative;
}
.item-active::before {
    content: '';
    position: absolute;
    top: 0; bottom: 0; left: -2px;
    width: 2px;
    box-shadow: 0 0 12px color-mix(in oklab, var(--accent) 50%, transparent);
    pointer-events: none;
}
```

---

## 15. Change log for this document

- **Initial version.** Captures the design system as implemented on the utterbar.com marketing site. Written to be handed to the team building the desktop app so the two read as the same product.
- **Logo redesigned.** Replaced the previous "pill with top-LED + waveform-below" with the new "pill with white border + white hole-punch + internal waveform" glyph. See §10.1. Old `mark.svg` / `wave.svg` geometry is retired.
- **Speech-bubble tail flipped.** The central waveform element's tail now points down-**right** (`M 74 10 A 2 2 0 0 1 78 10 V 50 L 74 44 Z`) — speech projecting forward. Bar x-positions shifted to `44, 54, 64, 74, 84, 94, 104`.
- **Buttons swapped.** CTA is now the white-filled pill with cyan neon border + cyan dot + dark text (maximum prominence). Ghost is the dark logo pill with white border + white dot + white text (quieter, matches the brand mark). Both share the `--pill-*` tokens with the logo.
