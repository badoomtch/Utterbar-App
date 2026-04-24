import { useEffect, useRef } from 'react';

/**
 * A lightweight canvas waveform driven by incoming RMS/peak values. We retain a rolling
 * buffer and redraw on animation frames, so smoothness doesn't depend on event cadence.
 */
export function Waveform({
  rms,
  peak,
  active,
}: {
  rms: number;
  peak: number;
  active: boolean;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const bufferRef = useRef<number[]>([]);
  const rafRef = useRef<number>();

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const dpr = window.devicePixelRatio || 1;

    const resize = () => {
      const { clientWidth, clientHeight } = canvas;
      canvas.width = Math.max(1, clientWidth * dpr);
      canvas.height = Math.max(1, clientHeight * dpr);
    };
    resize();
    const ro = new ResizeObserver(resize);
    ro.observe(canvas);

    const ctx = canvas.getContext('2d');
    const draw = () => {
      if (!ctx || !canvas) return;
      const w = canvas.width;
      const h = canvas.height;
      ctx.clearRect(0, 0, w, h);

      const buf = bufferRef.current;
      const bars = Math.floor(w / (4 * dpr));
      while (buf.length < bars) buf.unshift(0);
      while (buf.length > bars) buf.shift();

      // `--accent` is now a literal hex (post-design-system rewrite), not an
      // HSL triple. We read it straight and let globalAlpha below carry the
      // per-bar opacity — wrapping it in hsl()/rgb() functions would produce
      // invalid CSS and the canvas would silently fall back to black fill.
      const accent = getComputedStyle(document.documentElement).getPropertyValue('--accent').trim();
      ctx.fillStyle = accent || '#22D3EE';

      for (let i = 0; i < bars; i += 1) {
        const v = buf[i] ?? 0;
        const amp = Math.min(1, v * 2.8);
        const barH = Math.max(1 * dpr, amp * h * 0.85);
        const x = i * 4 * dpr;
        const y = (h - barH) / 2;
        ctx.globalAlpha = active ? 0.4 + amp * 0.6 : 0.2;
        ctx.fillRect(x, y, 2 * dpr, barH);
      }
      rafRef.current = requestAnimationFrame(draw);
    };
    rafRef.current = requestAnimationFrame(draw);
    return () => {
      ro.disconnect();
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
    };
  }, [active]);

  // Append current sample per render; hook reads the ref so we don't redraw on every frame via state.
  useEffect(() => {
    const buf = bufferRef.current;
    const sample = Math.max(rms, peak * 0.6);
    buf.push(sample);
    if (buf.length > 512) buf.shift();
  }, [rms, peak]);

  return <canvas ref={canvasRef} className="h-full w-full" />;
}
