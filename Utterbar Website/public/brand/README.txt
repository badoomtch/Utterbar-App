Utterbar — Brand Kit
====================

Contents
--------
SVG (vector, scale to any size):
  utterbar-mark.svg            The glyph only. Transparent background.
  utterbar-wordmark.svg        Wordmark for dark backgrounds.
  utterbar-wordmark-light.svg  Wordmark for light backgrounds.
  utterbar-logo-dark.svg       Full lockup (glyph + wordmark) for dark backgrounds.
  utterbar-logo-light.svg      Full lockup for light backgrounds.
  utterbar-app-icon.svg        The squircle app icon (256x256).

PNG (raster, inside /png):
  utterbar-mark-{512,1024,2048}.png           Transparent background.
  utterbar-wordmark-{512,1024,2048}.png       Baked on #0A0E17 (dark).
  utterbar-wordmark-light-{512,1024,2048}.png Baked on #F5F5F4 (light).
  utterbar-logo-dark-{1024,2048,4096}.png     Baked on #0A0E17.
  utterbar-logo-light-{1024,2048,4096}.png    Baked on #F5F5F4.
  utterbar-app-icon-{256,512,1024}.png        Transparent around the squircle.

Prefer the SVGs for web and print. Use the PNGs for places that don't accept SVG
(social profile pictures, some CMSes, presentation slides).

Typography
----------
The wordmark is set in Instrument Sans (Variable), weight 600, letter-spacing -0.04em.
Instrument Sans is an SIL Open Font License typeface: https://fonts.google.com/specimen/Instrument+Sans

The PNGs have the wordmark baked in as pixels, so they reproduce identically everywhere.
The SVGs reference the font by name with a system-sans fallback — for pixel-perfect
reproduction, install Instrument Sans locally or outline the text in your vector editor.

Colours
-------
Accent cyan       #22D3EE
Accent cyan hi    #67E8F9
Accent cyan deep  #0891B2
Wave gradient     #00F6FF -> #00A3FF (top -> bottom)
Pill body         #223042 -> #111827 (top -> bottom)
Pill border       #FFFFFF
Background dark   #0A0E17
Surface dark      #10151F
Foreground hi     #FFFFFF
Foreground        #E8E8EB
Muted             #8A8A94
Light surface     #F5F5F4

Clear space & minimum size
--------------------------
Keep clear space around the lockup equal to the height of the hole-punch circle
inside the pill glyph (roughly 1/4 of the glyph height).
Minimum width for the full lockup: 160px on screen, 40mm in print.
Minimum width for the glyph alone: 32px.

Do
--
- Place the logo on one of the brand backgrounds, or on clean black/white.
- Keep the proportions of the supplied files.
- Use the dark lockup on dark surfaces, the light lockup on light surfaces.

Don't
-----
- Recolour the glyph or the waveform gradient.
- Stretch, skew, rotate, or add drop shadows.
- Place the logo on busy imagery without a background plate.
- Recreate the wordmark in another typeface.

Questions
---------
hello@utterbar.com
