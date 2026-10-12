<?php

namespace App\Http\Controllers;

use App\Support\LatestGitHubRelease;

class PageController extends Controller
{
    public function home(LatestGitHubRelease $latestGitHubRelease)
    {
        return view('pages.home', [
            'latestRelease' => (object) $latestGitHubRelease->get(),
        ]);
    }

    public function features() { return view('pages.features'); }
    public function howItWorks() { return view('pages.how-it-works'); }
    public function privacy() { return view('pages.privacy'); }
    public function pricing() { return view('pages.pricing'); }
    public function changelog() { return redirect()->away(config('site.github_releases_url')); }
    public function download() { return redirect()->away(config('site.github_latest_release_url')); }
    public function contact() { return view('pages.contact'); }
    public function roadmap() { return view('pages.roadmap'); }
    public function press() { return view('pages.press'); }
    public function blog()
    {
        return view('pages.blog.index', [
            'posts' => \App\Support\BlogPost::all(),
        ]);
    }

    public function blogShow(string $slug)
    {
        $post = \App\Support\BlogPost::findBySlug($slug);
        if (!$post) {
            abort(404);
        }
        return view('pages.blog.show', ['post' => $post]);
    }

    // Comparison / "alternative to" pages - high-intent commercial searches.
    public function compareIndex() { return view('pages.compare.index'); }
    public function compareWisprFlow() { return view('pages.compare.wispr-flow'); }
    public function compareDragon() { return view('pages.compare.dragon'); }
    public function compareVoiceAccess() { return view('pages.compare.voice-access'); }
    public function compareSuperwhisper() { return view('pages.compare.superwhisper'); }

    // Educational deep-dive: explains on-device speech recognition without selling.
    public function whatIsLocalDictation() { return view('pages.what-is-local-dictation'); }

    public function docsIndex() { return view('pages.docs.index'); }
    public function docsGettingStarted() { return view('pages.docs.getting-started'); }
    public function docsModes() { return view('pages.docs.modes'); }
    public function docsVocabulary() { return view('pages.docs.vocabulary'); }
    public function docsShortcuts() { return view('pages.docs.shortcuts'); }
    public function docsPrivacyControls() { return view('pages.docs.privacy-controls'); }
    public function docsTroubleshooting() { return view('pages.docs.troubleshooting'); }
    public function docsFaq() { return view('pages.docs.faq'); }
    public function docsItDeployment() { return view('pages.docs.it-deployment'); }

    public function legalPrivacy() { return view('pages.legal.privacy'); }
    public function legalTerms() { return view('pages.legal.terms'); }
    public function legalEula() { return view('pages.legal.eula'); }
}
