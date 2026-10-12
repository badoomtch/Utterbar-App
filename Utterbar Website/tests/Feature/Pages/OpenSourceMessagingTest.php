<?php

namespace Tests\Feature\Pages;

use PHPUnit\Framework\Attributes\DataProvider;
use Tests\TestCase;

class OpenSourceMessagingTest extends TestCase
{
    /**
     * @return array<string, array{0: string}>
     */
    public static function publicPageProvider(): array
    {
        return [
            'privacy' => ['/privacy'],
            'FAQ' => ['/docs/faq'],
            'comparison index' => ['/compare'],
            'Dragon comparison' => ['/compare/dragon'],
            'Voice Access comparison' => ['/compare/voice-access'],
            'local dictation article' => ['/blog/why-local-dictation'],
        ];
    }

    #[DataProvider('publicPageProvider')]
    public function test_public_pages_do_not_repeat_retired_commercial_claims(string $path): void
    {
        $response = $this->get($path);

        $response->assertOk();
        $response->assertDontSee('Utterbar charges £39.99', false);
        $response->assertDontSee('Utterbar is paid', false);
        $response->assertDontSee('Utterbar isn&#039;t open source', false);
        $response->assertDontSee('paid features stay paid', false);
        $response->assertDontSee('licence arrives by email', false);
        $response->assertDontSee('Request a source review', false);
        $response->assertDontSee('we run it on your PC and charge once', false);
        $response->assertDontSee('Free Windows feature vs paid app', false);
        $response->assertDontSee('direct GBP pricing', false);
        $response->assertDontSee('Direct GBP purchase', false);
        $response->assertDontSee('signed updates every few weeks', false);
    }

    public function test_privacy_and_faq_link_to_the_public_source(): void
    {
        $this->get('/privacy')
            ->assertOk()
            ->assertSee('Free and open source', false)
            ->assertSee(config('site.github_repository_url'), false);

        $this->get('/docs/faq')
            ->assertOk()
            ->assertSee('open source under the MIT licence', false)
            ->assertSee(config('site.github_repository_url'), false);
    }
}
