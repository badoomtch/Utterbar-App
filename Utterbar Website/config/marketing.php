<?php

/**
 * Mirror of marketing/remotion/asset-manifest.json.
 *
 * The source of truth is that JSON file — this is a PHP copy so Blade
 * components can reference it without reaching into the desktop repo.
 * When the Remotion pipeline re-renders and dimensions / durations /
 * alt text change, update both. 2026-04-24 regenerate moved all videos
 * to 1440x900 and all stills to 2200x1400.
 */

return [
    // Assets are served directly from public/videos, public/posters, public/stills.
    // Env-overridable so a future CDN mirror is a one-line flip if needed.
    'base_url' => env('MARKETING_CDN_BASE', ''),

    'videos' => [
        'hero-pill-loop' => [
            'title'            => 'Floating bar — idle to inserted loop',
            'duration_seconds' => 5,
            'width'            => 1600,
            'height'           => 520,
            'alt'              => "Utterbar's floating bar cycling from idle through listening, processing, and 'Inserted'.",
            'loops'            => true,
        ],
        'feature-press-speak-done' => [
            'title'            => 'Press. Speak. Done.',
            'duration_seconds' => 4,
            'width'            => 1600,
            'height'           => 520,
            'alt'              => 'Close-framed loop of the pill moving from Ready to Inserted.',
            'loops'            => true,
        ],
        'feature-works-anywhere' => [
            'title'            => 'Types wherever your cursor is',
            'duration_seconds' => 7,
            'width'            => 1440,
            'height'           => 900,
            'alt'              => 'The pill above a plain text field; text streams into the field as the user dictates.',
            'loops'            => false,
        ],
        'feature-modes' => [
            'title'            => 'Modes — one voice, three shapes',
            'duration_seconds' => 7,
            'width'            => 1440,
            'height'           => 900,
            'alt'              => 'A raw transcript at the top; Voice-to-text, Message, and Meeting panels populate below it.',
            'loops'            => false,
        ],
        'feature-system-audio' => [
            'title'            => 'Transcribe what your PC is playing',
            'duration_seconds' => 6,
            'width'            => 1440,
            'height'           => 900,
            'alt'              => 'Audio source selector switching from Microphone to System audio; live transcript fills below.',
            'loops'            => false,
        ],
        'feature-vocabulary' => [
            'title'            => 'Custom vocabulary + replacements',
            'duration_seconds' => 7,
            'width'            => 1440,
            'height'           => 900,
            'alt'              => 'Custom words and replacements populate; a sentence types, rewrites itself with the replacements firing.',
            'loops'            => false,
        ],
        'feature-local-privacy' => [
            'title'            => 'Local by default',
            'duration_seconds' => 6,
            'width'            => 1440,
            'height'           => 900,
            'alt'              => 'The real Privacy settings — three context toggles all off, plus the on-disk paths where data lives.',
            'loops'            => false,
        ],
        'feature-file-transcription' => [
            'title'            => 'Batch file transcription',
            'duration_seconds' => 7,
            'width'            => 1440,
            'height'           => 900,
            'alt'              => 'Files dropzone with a queue of three jobs moving through queued, running, and done.',
            'loops'            => false,
        ],
        'feature-history-retranscribe' => [
            'title'            => 'Reprocess any past dictation',
            'duration_seconds' => 7.5,
            'width'            => 1440,
            'height'           => 900,
            'alt'              => "A history entry's processed output re-runs through Meeting mode and rewrites into bullet points.",
            'loops'            => false,
        ],
        'feature-customise-bar' => [
            'title'            => 'Make the bar yours',
            'duration_seconds' => 6,
            'width'            => 1600,
            'height'           => 520,
            'alt'              => "A row of six accent swatches cycles, and the pill's LED + waveform change colour to match.",
            'loops'            => false,
        ],
    ],

    'stills' => [
        'press-01-floating-bar-idle' => [
            'title'  => 'Floating bar — idle',
            'width'  => 2200,
            'height' => 1400,
            'alt'    => "Utterbar's floating pill in its idle Ready state, close-framed on a dark canvas.",
        ],
        'press-02-floating-bar-listening' => [
            'title'  => 'Floating bar — listening',
            'width'  => 2200,
            'height' => 1400,
            'alt'    => "Utterbar's floating pill mid-recording: 00:04 timer, seven-bar speech wave.",
        ],
        'press-03-main-shell-dashboard' => [
            'title'  => 'Main app — Dashboard',
            'width'  => 2200,
            'height' => 1180,
            'alt'    => 'Dashboard metrics and recent-activity card from the main Utterbar window.',
        ],
        'press-04-vocabulary' => [
            'title'  => 'Main app — Vocabulary',
            'width'  => 2200,
            'height' => 1280,
            'alt'    => 'Vocabulary and replacements cards from the main Utterbar window.',
        ],
        'press-05-file-transcription' => [
            'title'  => 'Main app — Files',
            'width'  => 2200,
            'height' => 1380,
            'alt'    => 'Files dropzone with a three-job queue.',
        ],
    ],
];
