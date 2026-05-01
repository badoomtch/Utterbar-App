<?php

use App\Http\Controllers\AccountController;
use App\Http\Controllers\AdminController;
use App\Http\Controllers\Api\LicenceApiController;
use App\Http\Controllers\Api\PricingApiController;
use App\Http\Controllers\Api\UpdaterApiController;
use App\Http\Controllers\BusinessEnquiryController;
use App\Http\Controllers\CheckoutController;
use App\Http\Controllers\ContactController;
use App\Http\Controllers\DownloadController;
use App\Http\Controllers\OgImageController;
use App\Http\Controllers\PageController;
use App\Http\Controllers\SignInController;
use App\Http\Controllers\StripeWebhookController;
use App\Http\Controllers\TipController;
use App\Http\Controllers\UpdatesController;
use Illuminate\Support\Facades\Route;

Route::get('/sitemap.xml', [\App\Http\Controllers\SitemapController::class, 'index'])->name('sitemap');

// Per-page OG image generator. Title + kicker arrive via query string;
// rendered output cached on disk by hash so repeat scrapes are static.
// Per-page OG image generator. Title + kicker arrive via query string;
// rendered output cached on disk by hash so repeat scrapes are static.
// Rate-limited so a scripted attacker can't force the GD pipeline to
// fill the Cloudways droplet's disk with PNGs (each unique title +
// kicker pair produces a permanent ~30-50 KB file). Cache itself is
// LRU-capped inside the controller.
Route::get('/og/image', [OgImageController::class, 'show'])
    ->middleware('throttle:30,1')
    ->name('og.image');

Route::get('/', [PageController::class, 'home'])->name('home');
Route::get('/features', [PageController::class, 'features'])->name('features');
Route::get('/how-it-works', [PageController::class, 'howItWorks'])->name('how-it-works');
Route::get('/privacy', [PageController::class, 'privacy'])->name('privacy');
Route::get('/pricing', [PageController::class, 'pricing'])->name('pricing');
Route::get('/changelog', [PageController::class, 'changelog'])->name('changelog');
Route::get('/download', [PageController::class, 'download'])->name('download');
Route::get('/contact',  [PageController::class, 'contact'])->name('contact');
Route::post('/api/contact', [ContactController::class, 'store'])
    ->middleware('throttle:6,1')
    ->name('contact.store');
Route::get('/roadmap', [PageController::class, 'roadmap'])->name('roadmap');
Route::get('/press', [PageController::class, 'press'])->name('press');
Route::get('/blog',          [PageController::class, 'blog'])->name('blog');
Route::get('/blog/{slug}',   [PageController::class, 'blogShow'])
    ->where('slug', '[a-z0-9-]+')
    ->name('blog.show');

Route::get('/what-is-local-dictation', [PageController::class, 'whatIsLocalDictation'])->name('what-is-local-dictation');

// Business / volume sales path. The landing page is editorial; the
// /business/contact form posts to /api/business/enquiry. Kept on
// separate routes so the typed B2B leads don't drown in general
// support volume.
Route::get('/business', [PageController::class, 'business'])->name('business');
Route::get('/business/contact', [BusinessEnquiryController::class, 'show'])->name('business.contact');
Route::post('/api/business/enquiry', [BusinessEnquiryController::class, 'store'])
    ->middleware('throttle:4,1')
    ->name('business.store');

Route::prefix('compare')->name('compare.')->group(function () {
    Route::get('/', [PageController::class, 'compareIndex'])->name('index');
    Route::get('/wispr-flow', [PageController::class, 'compareWisprFlow'])->name('wispr-flow');
});

Route::prefix('docs')->name('docs.')->group(function () {
    Route::get('/', [PageController::class, 'docsIndex'])->name('index');
    Route::get('/getting-started', [PageController::class, 'docsGettingStarted'])->name('getting-started');
    Route::get('/modes', [PageController::class, 'docsModes'])->name('modes');
    Route::get('/vocabulary', [PageController::class, 'docsVocabulary'])->name('vocabulary');
    Route::get('/shortcuts', [PageController::class, 'docsShortcuts'])->name('shortcuts');
    Route::get('/privacy-controls', [PageController::class, 'docsPrivacyControls'])->name('privacy-controls');
    Route::get('/troubleshooting', [PageController::class, 'docsTroubleshooting'])->name('troubleshooting');
    Route::get('/faq', [PageController::class, 'docsFaq'])->name('faq');
    Route::get('/it-deployment', [PageController::class, 'docsItDeployment'])->name('it-deployment');
});

Route::prefix('legal')->name('legal.')->group(function () {
    Route::get('/privacy', [PageController::class, 'legalPrivacy'])->name('privacy');
    Route::get('/terms', [PageController::class, 'legalTerms'])->name('terms');
    Route::get('/eula', [PageController::class, 'legalEula'])->name('eula');
});

Route::get('/thanks', [PageController::class, 'thanks'])->name('thanks');
Route::post('/api/checkout/create-session', [CheckoutController::class, 'createSession'])
    ->middleware('throttle:12,1')
    ->name('checkout.create');
// Volume self-serve checkout (B2B). Same throttle as single-user since
// the abuse profile is identical: a request creates a Stripe Session
// (a real network call) and returns a URL.
Route::post('/api/checkout/create-volume-session', [CheckoutController::class, 'createVolumeSession'])
    ->middleware('throttle:12,1')
    ->name('checkout.create.volume');
Route::post('/api/stripe/webhook', [StripeWebhookController::class, 'handle'])
    ->name('stripe.webhook');

// Magic-link auth
Route::get('/sign-in',               [SignInController::class, 'show'])->name('sign-in.show');
Route::post('/sign-in',              [SignInController::class, 'sendLink'])
    ->middleware('throttle:5,10')
    ->name('sign-in.send');
Route::get('/sign-in/{token}',       [SignInController::class, 'consume'])
    ->where('token', '[A-Za-z0-9]{64}')
    ->name('sign-in.consume');
Route::post('/sign-out',             [SignInController::class, 'signOut'])->name('sign-out');

// Authenticated account portal
Route::middleware('auth')->group(function () {
    Route::get('/account',                                             [AccountController::class, 'show'])->name('account');
    Route::post('/account/licences/{id}/resend',                       [AccountController::class, 'resendLicence'])
        ->whereNumber('id')
        ->name('account.licences.resend');
    Route::post('/account/licences/{id}/devices/{fingerprint}/revoke', [AccountController::class, 'deactivateDevice'])
        ->whereNumber('id')
        ->where('fingerprint', '[a-f0-9]{64}')
        ->name('account.licences.devices.revoke');
});

// Licence API — called by the Tauri app. CSRF-exempt (see bootstrap/app.php).
Route::prefix('api/licence')->group(function () {
    Route::post('fetch',      [LicenceApiController::class, 'fetch'])     ->middleware('throttle:20,10');
    Route::post('activate',   [LicenceApiController::class, 'activate'])  ->middleware('throttle:30,10');
    Route::post('deactivate', [LicenceApiController::class, 'deactivate'])->middleware('throttle:30,10');
    Route::post('heartbeat',  [LicenceApiController::class, 'heartbeat']) ->middleware('throttle:60,60');
});

// Pricing + updater endpoints — called by the Tauri app's in-app upgrade
// modal (pricing.rs) and auto-update checker (updater.rs). Public,
// unauthenticated, rate-limited.
Route::get('/api/pricing',        [PricingApiController::class, 'show'])
    ->middleware('throttle:60,1')
    ->name('api.pricing');

// Tip jar — opens a Stripe Checkout Session for a voluntary tip. The
// POST is form-driven from the tip-pills on home + download. The GET
// variant supports the desktop app's "Donate" surface (Settings →
// Licence on Free) which opens this URL in the user's default browser.
// Both paths converge on the same controller action; the GET handler
// reads `amount` from the query string instead of the POST body.
// Hidden when STRIPE_TIP_PRICE_ID is unset.
// Tip-jar throttle: real users click once or twice. The previous
// 30-per-10-minutes window was generous enough that a botnet could
// churn Stripe Checkout sessions and trip Stripe's "abuse pattern"
// detection on the account. 6 per minute is plenty for genuine use
// and tightens the screw on automated abuse.
Route::post('/tip/checkout',      [TipController::class, 'checkout'])
    ->middleware('throttle:6,1')
    ->name('tip.checkout');
Route::get('/tip/start',          [TipController::class, 'start'])
    ->middleware('throttle:6,1')
    ->name('tip.start');
Route::get('/tip/thanks',         [TipController::class, 'thanks'])
    ->name('tip.thanks');
Route::get('/api/updater/check',  [UpdaterApiController::class, 'check'])
    ->middleware('throttle:60,1')
    ->name('api.updater.check');

// Public download endpoint. Logs a unique-per-day-per-IP-per-UA event
// then 302s the user to the R2 download URL. Both /download/latest and
// /download/<version> are served by DownloadController.
Route::get('/download/latest',           [DownloadController::class, 'latest'])
    ->middleware('throttle:30,1')
    ->name('download.latest');
Route::get('/download/version/{version}', [DownloadController::class, 'version'])
    ->where('version', '\d+\.\d+\.\d+')
    ->middleware('throttle:30,1')
    ->name('download.version');

// Admin - email allow-list via ADMIN_EMAILS env var.
Route::middleware(['auth', 'admin'])->prefix('admin')->name('admin.')->group(function () {
    Route::get('/', [AdminController::class, 'index'])->name('index');

    // Releases.
    Route::get('/releases',          [\App\Http\Controllers\Admin\ReleaseController::class, 'index'])->name('releases.index');
    Route::get('/releases/create',   [\App\Http\Controllers\Admin\ReleaseController::class, 'create'])->name('releases.create');
    Route::post('/releases',         [\App\Http\Controllers\Admin\ReleaseController::class, 'store'])->name('releases.store');
    Route::delete('/releases/{id}',  [\App\Http\Controllers\Admin\ReleaseController::class, 'destroy'])
        ->whereNumber('id')
        ->name('releases.destroy');

    // Licences.
    Route::get('/licences',              [\App\Http\Controllers\Admin\LicenceController::class, 'index'])->name('licences.index');
    Route::get('/licences/create',       [\App\Http\Controllers\Admin\LicenceController::class, 'create'])->name('licences.create');
    Route::post('/licences',             [\App\Http\Controllers\Admin\LicenceController::class, 'store'])->name('licences.store');
    Route::get('/licences/{id}',         [\App\Http\Controllers\Admin\LicenceController::class, 'show'])
        ->whereNumber('id')->name('licences.show');
    // Managed-fleet licence file delivery. GET because IT admins drive
    // this from a browser; the binary is served as application/octet-
    // stream with a stable filename (utterbar-licence-<id>.bin).
    // Admin auth is enforced by the surrounding middleware group.
    Route::get('/licences/{id}/download', [\App\Http\Controllers\Admin\LicenceController::class, 'download'])
        ->whereNumber('id')->name('licences.download');
    // Air-gapped activation upload. Operator uploads the JSON file
    // the desktop wrote on the offline machine; the controller
    // validates, enforces max_devices, creates an offline=true
    // activation row. The admin then downloads licence.bin via the
    // route above.
    Route::post('/licences/{id}/activations/upload',
        [\App\Http\Controllers\Admin\LicenceController::class, 'uploadActivationRequest'])
        ->whereNumber('id')
        ->name('licences.activations.upload');
    Route::post('/licences/{id}/revoke',   [\App\Http\Controllers\Admin\LicenceController::class, 'revoke'])
        ->whereNumber('id')->name('licences.revoke');
    Route::post('/licences/{id}/unrevoke', [\App\Http\Controllers\Admin\LicenceController::class, 'unrevoke'])
        ->whereNumber('id')->name('licences.unrevoke');
    Route::post('/licences/{id}/resend',   [\App\Http\Controllers\Admin\LicenceController::class, 'resend'])
        ->whereNumber('id')->name('licences.resend');
    Route::post('/licences/{id}/remint',   [\App\Http\Controllers\Admin\LicenceController::class, 'remint'])
        ->whereNumber('id')->name('licences.remint');
    Route::post('/licences/{id}/devices/{fingerprint}/deactivate',
        [\App\Http\Controllers\Admin\LicenceController::class, 'deactivateDevice'])
        ->whereNumber('id')
        ->where('fingerprint', '[a-f0-9]{64}')
        ->name('licences.devices.deactivate');

    // Subscribers.
    Route::get('/subscribers',                [\App\Http\Controllers\Admin\SubscriberController::class, 'index'])->name('subscribers.index');
    Route::post('/subscribers/{id}/unsubscribe', [\App\Http\Controllers\Admin\SubscriberController::class, 'unsubscribe'])
        ->whereNumber('id')->name('subscribers.unsubscribe');

    // Contacts.
    Route::get('/contacts',       [\App\Http\Controllers\Admin\ContactController::class, 'index'])->name('contacts.index');
    Route::get('/contacts/{id}',  [\App\Http\Controllers\Admin\ContactController::class, 'show'])
        ->whereNumber('id')->name('contacts.show');

    // Business enquiries — sales leads from /business/contact. Kept
    // separate from the general /admin/contacts list so the operator
    // can triage them on different timelines.
    Route::get('/business',                 [\App\Http\Controllers\Admin\BusinessEnquiryController::class, 'index'])->name('business.index');
    Route::get('/business/{id}',            [\App\Http\Controllers\Admin\BusinessEnquiryController::class, 'show'])
        ->whereNumber('id')->name('business.show');
    Route::post('/business/{id}/status',    [\App\Http\Controllers\Admin\BusinessEnquiryController::class, 'updateStatus'])
        ->whereNumber('id')->name('business.status');

    // Analytics — unique downloads per release, country split, daily trend.
    Route::get('/analytics',      [\App\Http\Controllers\Admin\AnalyticsController::class, 'index'])->name('analytics.index');

    // Tips — voluntary "Tip the developer" Stripe payments. Total raised,
    // recent tips, daily chart.
    Route::get('/tips',           [\App\Http\Controllers\Admin\TipController::class, 'index'])->name('tips.index');
});

// Release update subscribers — "email me when there's a new release".
// /api/updates/subscribe is called from the Tauri app; confirm + unsubscribe
// are GETs so email clients can pre-fetch safely (they don't mutate state
// beyond marking the token as used).
Route::post('/api/updates/subscribe', [UpdatesController::class, 'subscribe'])
    ->middleware('throttle:5,10')
    ->name('updates.subscribe');
// Two-step confirm: GET renders a button page, POST commits the
// confirmation. Avoids email-prefetcher (Outlook safe-link, corporate
// URL scanners) auto-confirming subscriptions before the user clicks.
Route::get('/updates/confirm',     [UpdatesController::class, 'confirmPage'])->name('updates.confirm');
Route::post('/updates/confirm',    [UpdatesController::class, 'confirmPost'])->name('updates.confirm.post');

// Unsubscribe stays GET per RFC 8058 — that's the standard for
// email "list-unsubscribe" links and aggressive prefetching there is
// a feature, not a bug (auto-unsubs from broken inboxes).
Route::get('/updates/unsubscribe', [UpdatesController::class, 'unsubscribe'])->name('updates.unsubscribe');

// Headless admin API — bearer-token-authed (ADMIN_API_TOKEN env var).
// Used by the app-repo release script so the publish flow can run
// end-to-end without touching the admin form.
//
// Throttle tightened from 30/min to 6/min after the v0.8.x security
// audit: real publish traffic is at most one request every few weeks,
// 6 per minute is plenty for genuine retries and aggressive against
// any token-leak abuse pattern. The middleware itself logs every
// request for an audit trail (see App\Http\Middleware\AdminApiToken).
Route::middleware(['admin.api', 'throttle:6,1'])->prefix('api/admin')->group(function () {
    Route::post('releases',            [\App\Http\Controllers\Api\AdminReleaseApiController::class, 'store']);
    Route::delete('releases/{id}',     [\App\Http\Controllers\Api\AdminReleaseApiController::class, 'destroy'])
        ->whereNumber('id');
});
