<?php

return [
    'name'     => 'Utterbar',
    'tagline'  => 'Dictation that stays on your PC.',
    'domain'   => 'utterbar.com',
    // Informational only. GitHub Releases is the source of truth for builds.
    'version'  => '0.25.1',
    'github_releases_url' => 'https://github.com/badoomtch/Utterbar-App/releases',
    'github_latest_release_url' => 'https://github.com/badoomtch/Utterbar-App/releases/latest',

    'email' => [
        // 2026-04-27: outbound email moved off utterbar.com to the studio
        // domain ddraig.dev (Resend-verified, DKIM+SPF passing). The website
        // canonical domain is still utterbar.com — only the email address
        // changed. Replies go to the same inbox.
        'contact' => 'hello@ddraig.dev',
    ],

    'description' => 'Utterbar is a Windows-native, local-only dictation app. Press a hotkey, speak, type anywhere. Offline, private, fast.',
];
