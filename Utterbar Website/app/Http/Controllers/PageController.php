<?php

namespace App\Http\Controllers;

class PageController extends Controller
{
    public function home() {
        $latest = \App\Models\Release::stable()->orderByDesc('published_at')->first();
        return view('pages.home', ['latestRelease' => $latest]);
    }
    public function features() { return view('pages.features'); }
    public function howItWorks() { return view('pages.how-it-works'); }
    public function privacy() { return view('pages.privacy'); }
    public function pricing() { return view('pages.pricing'); }
    public function changelog() { return view('pages.changelog'); }
    public function download() {
        $releases = \App\Models\Release::stable()->orderByDesc('published_at')->get();
        return view('pages.download', [
            'releasesList' => $releases,
            'latest'       => $releases->first(),
        ]);
    }
    public function contact() { return view('pages.contact'); }
    public function thanks()  { return view('pages.thanks'); }
    public function roadmap() { return view('pages.roadmap'); }
    public function press() { return view('pages.press'); }
    public function waitlist() { return view('pages.waitlist'); }
    public function blog() { return view('pages.blog'); }

    public function docsIndex() { return view('pages.docs.index'); }
    public function docsGettingStarted() { return view('pages.docs.getting-started'); }
    public function docsModes() { return view('pages.docs.modes'); }
    public function docsVocabulary() { return view('pages.docs.vocabulary'); }
    public function docsShortcuts() { return view('pages.docs.shortcuts'); }
    public function docsPrivacyControls() { return view('pages.docs.privacy-controls'); }
    public function docsTroubleshooting() { return view('pages.docs.troubleshooting'); }
    public function docsFaq() { return view('pages.docs.faq'); }

    public function legalPrivacy() { return view('pages.legal.privacy'); }
    public function legalTerms() { return view('pages.legal.terms'); }
    public function legalEula() { return view('pages.legal.eula'); }
}
