import './bootstrap';
import Alpine from 'alpinejs';
import collapse from '@alpinejs/collapse';

Alpine.plugin(collapse);

/**
 * <x-video-embed> behaviour:
 *  - Respects prefers-reduced-motion (renders <img> poster only)
 *  - Uses IntersectionObserver to play when >=50% visible, pause when out
 *  - Hover starts playback immediately; mouseleave pauses LOOPING videos only
 *    (non-loop videos keep playing so the user can watch the beat finish)
 */
Alpine.data('ubVideoEmbed', () => ({
    reducedMotion: false,
    observer: null,
    _hovered: false,

    init() {
        this.reducedMotion = window.matchMedia?.('(prefers-reduced-motion: reduce)').matches === true;

        if (this.reducedMotion) return;

        // Wait for the <video> ref to mount (it's inside an x-if template).
        this.$nextTick(() => {
            const v = this.$refs.video;
            if (!v) return;

            this.observer = new IntersectionObserver((entries) => {
                entries.forEach((e) => {
                    if (e.isIntersecting && e.intersectionRatio >= 0.5) {
                        // Only autoplay if loop — non-loop beats should start on interaction
                        if (v.loop) v.play().catch(() => {});
                    } else {
                        v.pause();
                    }
                });
            }, { threshold: [0, 0.5, 1] });
            this.observer.observe(this.$el);
        });
    },

    playIfReady() {
        this._hovered = true;
        const v = this.$refs.video;
        if (v && !this.reducedMotion) v.play().catch(() => {});
    },

    pauseIfLoop() {
        this._hovered = false;
        const v = this.$refs.video;
        if (v && v.loop) v.pause();
    },

    destroy() {
        this.observer?.disconnect();
    },
}));

window.Alpine = Alpine;
Alpine.start();
