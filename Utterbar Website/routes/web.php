<?php

use App\Http\Controllers\Api\UpdaterApiController;
use App\Http\Controllers\ContactController;
use App\Http\Controllers\DownloadController;
use App\Http\Controllers\OgImageController;
use App\Http\Controllers\PageController;
use Illuminate\Support\Facades\Route;

Route::get('/sitemap.xml', [\App\Http\Controllers\SitemapController::class, 'index'])->name('sitemap');

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

Route::prefix('compare')->name('compare.')->group(function () {
    Route::get('/', [PageController::class, 'compareIndex'])->name('index');
    Route::get('/wispr-flow', [PageController::class, 'compareWisprFlow'])->name('wispr-flow');
    Route::get('/dragon', [PageController::class, 'compareDragon'])->name('dragon');
    Route::get('/voice-access', [PageController::class, 'compareVoiceAccess'])->name('voice-access');
    Route::get('/superwhisper', [PageController::class, 'compareSuperwhisper'])->name('superwhisper');
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

// Auto-updater feed — called by the desktop app's update checker
// (updater.rs). Public, unauthenticated, rate-limited. The feed is
// Ed25519-signed; the client verifies before downloading.
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

// Headless admin API — bearer-token-authed (ADMIN_API_TOKEN env var).
// Used by the app-repo release script so the publish flow can run
// end-to-end without touching an admin form. This is the only
// privileged surface that survives: the app is free and open source,
// but only releases signed with the operator's updater key are served.
//
// Throttle tightened to 6/min: real publish traffic is at most one
// request every few weeks. The middleware logs every request for an
// audit trail (see App\Http\Middleware\AdminApiToken).
Route::middleware(['admin.api', 'throttle:6,1'])->prefix('api/admin')->group(function () {
    Route::post('releases',            [\App\Http\Controllers\Api\AdminReleaseApiController::class, 'store']);
    Route::delete('releases/{id}',     [\App\Http\Controllers\Api\AdminReleaseApiController::class, 'destroy'])
        ->whereNumber('id');
});
