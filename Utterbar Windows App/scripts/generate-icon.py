"""Rasterise the Utterbar marks into the PNGs that Tauri's icon CLI + tray consume.

Produces three PNGs under src-tauri/icons/:
  - source.png          — the full mark (pill + waveform) on a transparent background.
                          Feed this into `pnpm tauri icon src-tauri/icons/source.png` to
                          fan out every Windows / installer size + icon.ico. This is
                          what renders as the taskbar / Start menu / installer icon —
                          no dark rounded-square tile.
  - source-square.png   — the same mark composited onto a dark rounded-square container.
                          Retained for Microsoft Store tile assets (StoreLogo.png etc.)
                          which need an opaque background. NOT used for the taskbar.
  - tray.png            — just the waveform bars on a transparent background, sized for
                          the system tray. Loaded directly by Tauri (see tauri.conf.json
                          `trayIcon.iconPath`).
"""

import io
import os

import resvg_py
from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
ICON_DIR = os.path.join(REPO, 'src-tauri', 'icons')
MARK_SVG = os.path.join(ICON_DIR, 'mark.svg')
WAVE_SVG = os.path.join(ICON_DIR, 'wave.svg')
SIZE = 1024
TRAY_SIZE = 256


def rasterise_svg(svg_path: str, size: int) -> Image.Image:
    """Render an SVG at `size` × `size` onto a transparent canvas."""
    with open(svg_path, 'r', encoding='utf-8') as f:
        svg = f.read()
    png_bytes = resvg_py.svg_to_bytes(svg_string=svg, width=size, height=size)
    if isinstance(png_bytes, list):
        png_bytes = bytes(png_bytes)
    return Image.open(io.BytesIO(png_bytes)).convert('RGBA')


def rounded_mask(w: int, h: int, radius: int) -> Image.Image:
    m = Image.new('L', (w, h), 0)
    ImageDraw.Draw(m).rounded_rectangle([0, 0, w, h], radius=radius, fill=255)
    return m


def dark_square_bg(size: int) -> Image.Image:
    """Vertical-gradient dark square, rounded to a squircle-ish radius."""
    base = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    grad = Image.new('RGBA', (size, size))
    px = grad.load()
    for y in range(size):
        t = y / max(size - 1, 1)
        r = int(20 + (8 - 20) * t)
        g = int(26 + (12 - 26) * t)
        b = int(36 + (18 - 36) * t)
        for x in range(size):
            px[x, y] = (r, g, b, 255)
    mask = rounded_mask(size, size, int(size * 0.22))
    base.paste(grad, (0, 0), mask)

    border = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    ImageDraw.Draw(border).rounded_rectangle(
        [1, 1, size - 2, size - 2],
        radius=int(size * 0.22),
        outline=(40, 50, 64, 255),
        width=3,
    )
    return Image.alpha_composite(base, border)


def build_square_icon() -> Image.Image:
    """Mark centred on a dark rounded-square container (Store tile asset)."""
    bg = dark_square_bg(SIZE)
    inset = int(SIZE * 0.12)
    inner = SIZE - inset * 2
    mark = rasterise_svg(MARK_SVG, inner)
    offset_x = (SIZE - mark.width) // 2
    offset_y = (SIZE - mark.height) // 2
    composed = bg.copy()
    composed.alpha_composite(mark, (offset_x, offset_y))
    return composed


def centre_on_square(mark: Image.Image, size: int, padding: float = 0.08) -> Image.Image:
    """Paste `mark` centred on a transparent square canvas with a % padding on each
    side. `tauri icon` requires the source PNG to be square, but our mark SVG has a
    non-square viewBox, so we composite it here."""
    canvas = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    inner = int(size * (1 - padding * 2))
    # Fit the mark into `inner` preserving aspect.
    scale = min(inner / mark.width, inner / mark.height)
    new_w = int(mark.width * scale)
    new_h = int(mark.height * scale)
    fitted = mark.resize((new_w, new_h), Image.LANCZOS)
    canvas.alpha_composite(fitted, ((size - new_w) // 2, (size - new_h) // 2))
    return canvas


def main() -> None:
    # 1. Taskbar / Start menu / installer icon.
    #    We now ship the waveform-only glyph (`wave.svg`) on transparent — the
    #    seven bright-cyan bars with the speech-bubble tail. Rationale:
    #
    #    The earlier pill mark shrank poorly at taskbar sizes (16-24px). At that
    #    scale the dark pill collapsed to "dark blob with a faint white ring",
    #    the hole-punch vanished, and the cyan waveform compressed to 3-4 px of
    #    ambiguous colour. Users saw a small dark rectangle instead of a
    #    recognisable Utterbar glyph.
    #
    #    The waveform-only version reads as bright cyan bars on transparent,
    #    which the Windows shell composites straight onto whatever taskbar tint
    #    the user has (dark bar, light bar, or an accent tint). The shell's own
    #    subtle backdrop shadow gives the bars enough separation on white that
    #    "disappears on light taskbar" isn't a real concern — at 32px bright
    #    cyan on white still clearly reads.
    raw = rasterise_svg(WAVE_SVG, SIZE)
    transparent = centre_on_square(raw, SIZE, padding=0.10)
    out = os.path.join(ICON_DIR, 'source.png')
    transparent.save(out)
    print('wrote', out, os.path.getsize(out), 'bytes')

    # 2. Dark-square mark — Microsoft Store tile assets need an opaque square
    #    background. Keep using the full pill on a dark squircle here because
    #    the Store tile is rendered at ≥71×71px, where the pill still reads
    #    clearly and the brand identity is reinforced on the tile.
    square = build_square_icon()
    out = os.path.join(ICON_DIR, 'source-square.png')
    square.save(out)
    print('wrote', out, os.path.getsize(out), 'bytes')

    # 3. Tray icon — waveform-only on transparent. The Windows system tray is
    #    a 16/24 px strip; the pill mark at those sizes was illegible (the user
    #    literally said "I can't make it out"). `wave.svg`'s bright cyan bars
    #    hold up at 16px the way any good glyph-only icon does. Padding is
    #    tight (2%) so we max out pixel density at tray sizes.
    tray_src = rasterise_svg(WAVE_SVG, TRAY_SIZE)
    tray = centre_on_square(tray_src, TRAY_SIZE, padding=0.02)
    out = os.path.join(ICON_DIR, 'tray.png')
    tray.save(out)
    print('wrote', out, os.path.getsize(out), 'bytes')


if __name__ == '__main__':
    main()
