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

    'stripe' => [
        // Feature flag. When false, the pricing page falls back to the waitlist CTA
        // and /api/checkout/* routes return 503. Flip to true once sandbox is configured.
        'enabled' => env('STRIPE_ENABLED', false),

        // Test keys during sandbox; flip to live keys when we cut over.
        'key'            => env('STRIPE_PUBLISHABLE_KEY'),
        'secret'         => env('STRIPE_SECRET_KEY'),
        'webhook_secret' => env('STRIPE_WEBHOOK_SECRET'),

        // Pro price: one-off, GBP. For sandbox we pass inline price_data so no
        // Stripe dashboard product is required. When we go live, set a real price ID
        // and the controller will prefer it over the inline amount.
        'pro_price_id'   => env('STRIPE_PRO_PRICE_ID'),
        'pro_amount'     => (int) env('STRIPE_PRO_AMOUNT', 4900),  // pence (£49.00)
        'pro_currency'   => env('STRIPE_PRO_CURRENCY', 'gbp'),

        // Whether to ask Stripe to collect VAT/tax automatically. Requires Stripe Tax
        // to be configured in the dashboard. Safe to leave off in sandbox.
        'automatic_tax'  => env('STRIPE_AUTOMATIC_TAX', false),
    ],

    'admin' => [
        // Email that receives operator notifications (new purchase, new waitlist
        // signup). Silent no-op if blank. See app/Services/Notify/AdminNotify.php.
        'notify_email' => env('ADMIN_NOTIFY_EMAIL'),
    ],

    'licence' => [
        // Base64-encoded 64-byte Ed25519 secret key (libsodium format).
        // Generated once via `php artisan licence:keygen`.
        'private_key_b64' => env('LICENCE_PRIVATE_KEY_B64'),
        // Base64-encoded 32-byte Ed25519 public key. Also baked into the Tauri app.
        'public_key_b64'  => env('LICENCE_PUBLIC_KEY_B64'),
        // Integer SKU identifier baked into signed payload.
        'sku_id'          => (int) env('LICENCE_SKU_ID', 1),
        // SKU string used by the checkout metadata.
        'sku_string'      => env('LICENCE_SKU_STRING', 'utterbar-pro-lifetime'),
        'max_devices'     => (int) env('LICENCE_MAX_DEVICES', 3),
        // Payload version byte. Bump when we rotate signing keys.
        'payload_version' => (int) env('LICENCE_PAYLOAD_VERSION', 1),
    ],

];
