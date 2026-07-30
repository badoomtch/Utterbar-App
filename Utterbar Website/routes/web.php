<?php

use App\Http\Controllers\ContactController;
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
