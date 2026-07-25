<?php

return [

    /*
    |--------------------------------------------------------------------------
    | Third Party Services
    |--------------------------------------------------------------------------
    |
    | This file is for storing the credentials for third party services such
    | as Mailgun, Postmark, AWS and more. This file provides the de facto
    | location for this type of information, allowing packages to have
    | a conventional file to locate the various service credentials.
    |
    */

    'postmark' => [
        'key' => env('POSTMARK_API_KEY'),
    ],

    'resend' => [
        'key' => env('RESEND_API_KEY'),
    ],

    'ses' => [
        'key' => env('AWS_ACCESS_KEY_ID'),
        'secret' => env('AWS_SECRET_ACCESS_KEY'),
        'region' => env('AWS_DEFAULT_REGION', 'us-east-1'),
    ],

    'slack' => [
        'notifications' => [
            'bot_user_oauth_token' => env('SLACK_BOT_USER_OAUTH_TOKEN'),
            'channel' => env('SLACK_BOT_USER_DEFAULT_CHANNEL'),
        ],
    ],

    'admin' => [
        // Email that receives operator notifications (new contact message,
        // new business enquiry). Silent no-op if blank.
        // See app/Services/Notify/AdminNotify.php.
        'notify_email' => env('ADMIN_NOTIFY_EMAIL'),
    ],

    'updater' => [
        // Ed25519 keypair used to sign release-update metadata returned by
        // /api/updater/check. Distinct from the licence keypair on purpose:
        // a compromised licence key still can't push updates, and a
        // compromised updater key still can't mint licences. The private
        // key is operator-local — the publish script signs the canonical
        // metadata blob and POSTs the signature to the admin API. Only
        // the public key needs to live on the server (for verification at
        // publish time + sanity-check tooling). The desktop client embeds
        // the public key at compile time.
        //
        // Generated once via `php artisan updater:keygen`.
        'public_key_b64' => env('UPDATER_PUBLIC_KEY_B64'),
    ],

];
