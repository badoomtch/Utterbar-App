<?php

use App\Http\Controllers\AccountController;
use App\Http\Controllers\AdminController;
use App\Http\Controllers\Api\LicenceApiController;
use App\Http\Controllers\Api\PricingApiController;
use App\Http\Controllers\Api\UpdaterApiController;
use App\Http\Controllers\CheckoutController;
use App\Http\Controllers\ContactController;
use App\Http\Controllers\PageController;
use App\Http\Controllers\SignInController;
use App\Http\Controllers\StripeWebhookController;
use App\Http\Controllers\UpdatesController;
use App\Http\Controllers\WaitlistController;
use Illuminate\Support\Facades\Route;

Route::get('/sitemap.xml', [\App\Http\Controllers\SitemapController::class, 'index'])->name('sitemap');

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
Route::get('/waitlist', [PageController::class, 'waitlist'])->name('waitlist');
Route::get('/blog', [PageController::class, 'blog'])->name('blog');

Route::prefix('docs')->name('docs.')->group(function () {
    Route::get('/', [PageController::class, 'docsIndex'])->name('index');
    Route::get('/getting-started', [PageController::class, 'docsGettingStarted'])->name('getting-started');
    Route::get('/modes', [PageController::class, 'docsModes'])->name('modes');
    Route::get('/vocabulary', [PageController::class, 'docsVocabulary'])->name('vocabulary');
    Route::get('/shortcuts', [PageController::class, 'docsShortcuts'])->name('shortcuts');
    Route::get('/privacy-controls', [PageController::class, 'docsPrivacyControls'])->name('privacy-controls');
    Route::get('/troubleshooting', [PageController::class, 'docsTroubleshooting'])->name('troubleshooting');
    Route::get('/faq', [PageController::class, 'docsFaq'])->name('faq');
});

Route::prefix('legal')->name('legal.')->group(function () {
    Route::get('/privacy', [PageController::class, 'legalPrivacy'])->name('privacy');
    Route::get('/terms', [PageController::class, 'legalTerms'])->name('terms');
    Route::get('/eula', [PageController::class, 'legalEula'])->name('eula');
});

Route::post('/api/waitlist', [WaitlistController::class, 'store'])
    ->middleware('throttle:10,1')
    ->name('waitlist.store');

Route::get('/thanks', [PageController::class, 'thanks'])->name('thanks');
Route::post('/api/checkout/create-session', [CheckoutController::class, 'createSession'])
    ->middleware('throttle:12,1')
    ->name('checkout.create');
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
Route::get('/api/updater/check',  [UpdaterApiController::class, 'check'])
    ->middleware('throttle:60,1')
    ->name('api.updater.check');

// Admin — email allow-list via ADMIN_EMAILS env var.
Route::middleware(['auth', 'admin'])->group(function () {
    Route::get('/admin', [AdminController::class, 'index'])->name('admin');

    Route::get('/admin/releases',          [\App\Http\Controllers\Admin\ReleaseController::class, 'index'])->name('admin.releases.index');
    Route::get('/admin/releases/create',   [\App\Http\Controllers\Admin\ReleaseController::class, 'create'])->name('admin.releases.create');
    Route::post('/admin/releases',         [\App\Http\Controllers\Admin\ReleaseController::class, 'store'])->name('admin.releases.store');
    Route::delete('/admin/releases/{id}',  [\App\Http\Controllers\Admin\ReleaseController::class, 'destroy'])
        ->whereNumber('id')
        ->name('admin.releases.destroy');
});

// Release update subscribers — "email me when there's a new release".
// /api/updates/subscribe is called from the Tauri app; confirm + unsubscribe
// are GETs so email clients can pre-fetch safely (they don't mutate state
// beyond marking the token as used).
Route::post('/api/updates/subscribe', [UpdatesController::class, 'subscribe'])
    ->middleware('throttle:5,10')
    ->name('updates.subscribe');
Route::get('/updates/confirm',     [UpdatesController::class, 'confirm'])    ->name('updates.confirm');
Route::get('/updates/unsubscribe', [UpdatesController::class, 'unsubscribe'])->name('updates.unsubscribe');

// Headless admin API — bearer-token-authed (ADMIN_API_TOKEN env var).
// Used by the app-repo release script so app Claude can publish releases
// end-to-end without touching the admin form.
Route::middleware(['admin.api', 'throttle:30,1'])->prefix('api/admin')->group(function () {
    Route::post('releases',            [\App\Http\Controllers\Api\AdminReleaseApiController::class, 'store']);
    Route::delete('releases/{id}',     [\App\Http\Controllers\Api\AdminReleaseApiController::class, 'destroy'])
        ->whereNumber('id');
});
