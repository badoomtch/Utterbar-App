<?php

return [
    'name'     => 'Utterbar',
    'tagline'  => 'Dictation that stays on your PC.',
    'domain'   => 'utterbar.com',
    // Informational only. The authoritative source for "what version are
    // we on" is the `releases` table, surfaced via App\Models\Release and
    // consumed by every page that displays a version. This string is the
    // last-resort fallback used when that lookup yields nothing (a fresh
    // database, a deploy mishap, the docs layout rendering before the DB
    // is reachable). Bump on every public ship so the fallback stays
    // close to current truth and the status strip never claims something
    // wildly old. Currently tracking the latest in-tree app version
    // including beta-channel builds, since the website's `releases`
    // table is populated for both channels and a missed-DB fallback to
    // the most recent ship is closer to reality than the most recent
    // stable.
    'version'  => '0.14.1',

    'email' => [
        // 2026-04-27: outbound email moved off utterbar.com to the studio
        // domain ddraig.dev (Resend-verified, DKIM+SPF passing). The website
        // canonical domain is still utterbar.com — only the email address
        // changed. Replies go to the same inbox.
        'contact' => 'hello@ddraig.dev',
    ],

    'description' => 'Utterbar is a Windows-native, local-only dictation app. Press a hotkey, speak, type anywhere. Offline, private, fast.',
];
