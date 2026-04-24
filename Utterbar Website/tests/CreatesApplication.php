<?php

namespace Tests;

use Illuminate\Contracts\Console\Kernel;

/**
 * Boots the Laravel application for tests. Required by `TestCase` via the
 * `createApplication()` method that `BaseTestCase` calls in setUp.
 *
 * Kept trivial — phpunit.xml already wires the testing env (SQLite
 * :memory:, array mail, sync queues).
 */
trait CreatesApplication
{
    public function createApplication()
    {
        $app = require __DIR__ . '/../bootstrap/app.php';
        $app->make(Kernel::class)->bootstrap();
        return $app;
    }
}
