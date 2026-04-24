<?php

/**
 * Mirror of marketing/remotion/asset-manifest.json from the Orliq repo.
 *
 * The source of truth is that JSON file — this is a PHP copy so Blade
 * components can reference it without touching the other repo. When the
 * Remotion pipeline re-renders, the Orliq-side sync script pushes the new
 * blobs to R2 at the same keys, and (if a duration / alt / dimension
 * changes) we update this file by hand.
 *
 * CDN contract (matches Orliq/RELEASE.md + the R2 bucket layout):
 *   videos → https://dl.utterbar.com/marketing/videos/<id>.mp4
 *   posters → https://dl.utterbar.com/marketing/posters/<id>-poster.png
 *   stills → https://dl.utterbar.com/marketing/stills/<id>.png
 */

return [
    // Assets are served directly from public/videos, public/posters, public/stills.
    // Env-overridable so a future CDN mirror is a one-line flip if needed.
    'base_url' => env('MARKETING_CDN_BASE', ''),

    'videos' => [
        'hero-pill-loop' => [
            'title'            => 'Floating bar — idle to listening loop',
            'duration_seconds' => 6,
            'width'            => 1920,
            'height'           => 1080,
            'alt'              => "Utterbar's floating bar pill cycling through idle, warming, listening, and processing states.",
            'loops'            => true,
        ],
        'how-it-works-walkthrough' => [
            'title'            => 'How it works — press, speak, land in app',
            'duration_seconds' => 12,
            'width'            => 1920,
            'height'           => 1080,
            'alt'              => 'Three-beat walkthrough: pressing the shortcut, speaking into the floating bar, the transcript appearing inside Slack.',
            'loops'            => false,
        ],
        'feature-press-speak-done' => [
            'title'            => 'Press. Speak. Done.',
            'duration_seconds' => 5,
            'width'            => 1920,
            'height'           => 1080,
            'alt'              => "Tight state cycle on the floating bar under the tagline 'Press. Speak. Done.'",
            'loops'            => true,
        ],
        'feature-works-anywhere' => [
            'title'            => 'Works in every app',
            'duration_seconds' => 10,
            'width'            => 1920,
            'height'           => 1080,
            'alt'              => 'Utterbar dictating into Gmail, Slack, and Obsidian in sequence — the same pill, three different host apps.',
            'loops'            => false,
        ],
        'feature-modes' => [
            'title'            => 'Modes — one dictation, four voices',
            'duration_seconds' => 8,
            'width'            => 1920,
            'height'           => 1080,
            'alt'              => 'Raw transcript at the top, three mode panels (Voice-to-text, Message, Meeting) staggering in below.',
            'loops'            => false,
        ],
        'feature-system-audio' => [
            'title'            => 'System audio capture',
            'duration_seconds' => 7,
            'width'            => 1920,
            'height'           => 1080,
            'alt'              => 'A mock video-call window alongside the floating bar sourcing system audio, live transcript filling in beside it.',
            'loops'            => false,
        ],
        'feature-vocabulary' => [
            'title'            => 'Custom vocabulary + text replacements',
            'duration_seconds' => 8,
            'width'            => 1920,
            'height'           => 1080,
            'alt'              => 'Vocabulary grid with custom words and replacements, plus a before-and-after sample sentence at the bottom.',
            'loops'            => false,
        ],
        'feature-local-privacy' => [
            'title'            => 'Everything stays on your machine',
            'duration_seconds' => 8,
            'width'            => 1920,
            'height'           => 1080,
            'alt'              => 'Diagram: microphone to local Parakeet ASR to app, with the cloud path crossed out.',
            'loops'            => false,
        ],
        'feature-file-transcription' => [
            'title'            => 'Batch file transcription (Pro)',
            'duration_seconds' => 8,
            'width'            => 1920,
            'height'           => 1080,
            'alt'              => 'Files route with a dashed dropzone and a queue of three jobs running through to done.',
            'loops'            => false,
        ],
        'feature-history-retranscribe' => [
            'title'            => 'Reprocess any past dictation',
            'duration_seconds' => 9,
            'width'            => 1920,
            'height'           => 1080,
            'alt'              => 'History detail pane re-running a transcript through Meeting mode; the processed block rewrites itself into bullet points.',
            'loops'            => false,
        ],
        'feature-customise-bar' => [
            'title'            => 'Make it look like your desk',
            'duration_seconds' => 8,
            'width'            => 1920,
            'height'           => 1080,
            'alt'              => 'Floating bar flipping between Obsidian+Cyan, Graphite+Amber, and Paper+Violet themes.',
            'loops'            => false,
        ],
        'onboarding-walkthrough' => [
            'title'            => 'Set up in under two minutes',
            'duration_seconds' => 12,
            'width'            => 1920,
            'height'           => 1080,
            'alt'              => 'Onboarding window stepping through Welcome, privacy promise, and the first-dictation ready state.',
            'loops'            => false,
        ],
    ],

    'stills' => [
        'press-01-floating-bar-idle' => [
            'title'  => 'Floating bar — idle',
            'width'  => 2560,
            'height' => 1600,
            'alt'    => "Utterbar's floating pill in its idle Ready state, centred on a dark canvas.",
        ],
        'press-02-floating-bar-listening' => [
            'title'  => 'Floating bar — listening',
            'width'  => 2560,
            'height' => 1600,
            'alt'    => "Utterbar's floating pill mid-recording: red ripple, 00:04 timer, seven-bar speech wave.",
        ],
        'press-03-main-shell-dashboard' => [
            'title'  => 'Main app — Dashboard',
            'width'  => 2560,
            'height' => 1600,
            'alt'    => 'Utterbar main window on the Dashboard route, showing session metrics and recent activity.',
        ],
        'press-04-vocabulary' => [
            'title'  => 'Main app — Vocabulary',
            'width'  => 2560,
            'height' => 1600,
            'alt'    => 'Vocabulary route with custom words and text replacements populated.',
        ],
        'press-05-file-transcription' => [
            'title'  => 'Main app — Files',
            'width'  => 2560,
            'height' => 1600,
            'alt'    => 'Files route with dropzone and four queued / running / done jobs.',
        ],
    ],
];
