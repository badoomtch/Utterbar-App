<?php

namespace Tests;

use Illuminate\Foundation\Testing\TestCase as BaseTestCase;

abstract class TestCase extends BaseTestCase
{
    use CreatesApplication;

    protected function setUp(): void
    {
        parent::setUp();

        // Skip the @vite directive in tests. Without this, any view that
        // extends the main layout fails with "Vite manifest not found"
        // because we don't run `npm run build` before PHPUnit. No test
        // asserts on asset URLs anyway — the manifest is production UX
        // polish, not test surface.
        $this->withoutVite();
    }
}
