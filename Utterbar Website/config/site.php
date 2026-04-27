<?php

return [
    'name'     => 'Utterbar',
    'tagline'  => 'Dictation that stays on your PC.',
    'domain'   => 'utterbar.com',
    // Informational. Real release data is served from the `releases` table —
    // this string is only used as a fallback for templates (roadmap, schema.org)
    // when the latest-release lookup misses. Bump on every public stable ship
    // so the fallback isn't months behind reality.
    'version'  => '0.10.0',

    'email' => [
        // 2026-04-27: outbound email moved off utterbar.com to the studio
        // domain ddraig.dev (Resend-verified, DKIM+SPF passing). The website
        // canonical domain is still utterbar.com — only the email address
        // changed. Replies go to the same inbox.
        'contact' => 'hello@ddraig.dev',
    ],

    'description' => 'Utterbar is a Windows-native, local-only dictation app. Press a hotkey, speak, type anywhere. Offline, private, fast.',
];
