// One-shot renderer: converts the SVG brand assets in public/brand/ to PNG.
// Run with:  node scripts/render-brand-png.mjs
//
// Uses sharp (libvips + librsvg) because resvg-js silently drops gradient
// fills on <text> elements. sharp/librsvg handles them correctly.

import sharp from 'sharp';
import { readFileSync, writeFileSync, mkdirSync, unlinkSync, existsSync } from 'node:fs';

// density=300 gives libvips a high-DPI render target so the text anti-aliasing
// is sharp at all output sizes. We then resize to the final width.
const DENSITY = 300;

// Target widths per asset. The mark + app icon are square-ish, logos are wide.
// bg: baked-in background; null = transparent.
const TASKS = [
    { src: 'public/brand/utterbar-mark.svg',           widths: [512, 1024, 2048], bg: null },
    { src: 'public/brand/utterbar-wordmark.svg',       widths: [512, 1024, 2048], bg: '#0A0E17' },
    { src: 'public/brand/utterbar-wordmark-light.svg', widths: [512, 1024, 2048], bg: '#F5F5F4' },
    { src: 'public/brand/utterbar-logo-dark.svg',      widths: [1024, 2048, 4096], bg: '#0A0E17' },
    { src: 'public/brand/utterbar-logo-light.svg',     widths: [1024, 2048, 4096], bg: '#F5F5F4' },
    { src: 'public/brand/utterbar-app-icon.svg',       widths: [256, 512, 1024],   bg: null },
];

const OUT_DIR = 'public/brand/png';
mkdirSync(OUT_DIR, { recursive: true });

for (const t of TASKS) {
    const baseName = t.src.split('/').pop().replace('.svg', '');
    for (const w of t.widths) {
        const pipeline = sharp(t.src, { density: DENSITY }).resize(w);
        if (t.bg) {
            pipeline.flatten({ background: t.bg });
        }
        const out = `${OUT_DIR}/${baseName}-${w}.png`;
        const info = await pipeline.png().toFile(out);
        console.log(`${out}  (${(info.size / 1024).toFixed(1)} KB, ${info.width}x${info.height})`);
    }
}

// Clean up any lingering test files
for (const f of ['public/brand/_test.svg', 'public/brand/_test.png', 'public/brand/_test_sharp.png']) {
    if (existsSync(f)) unlinkSync(f);
}

console.log('\nDone.');
