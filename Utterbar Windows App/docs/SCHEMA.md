# Utterbar — Database Schema

SQLite database at `%APPDATA%/Utterbar/data/orliq.db`. Migrations managed via `refinery` and executed at startup.

## Migration 0001_init

```sql
-- Schema version
CREATE TABLE schema_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
INSERT INTO schema_meta(key, value) VALUES ('version', '1');

-- Modes
CREATE TABLE modes (
    id                 TEXT PRIMARY KEY,         -- ulid
    name               TEXT NOT NULL,
    icon               TEXT NOT NULL,            -- lucide icon key
    is_builtin         INTEGER NOT NULL DEFAULT 0,
    is_favourite       INTEGER NOT NULL DEFAULT 0,
    sort_order         INTEGER NOT NULL DEFAULT 0,
    asr_model_id       TEXT,                     -- NULL => inherit default
    target_language    TEXT,                     -- BCP-47 or NULL => auto
    post_processing    TEXT NOT NULL,            -- JSON { kind, prompt, template, … }
    context_rules      TEXT NOT NULL,            -- JSON { clipboard, selection, active_app }
    auto_activation    TEXT NOT NULL,            -- JSON [{ match: "by_exe"|"by_title"|"by_class", pattern }]
    shortcut_override  TEXT,                     -- JSON Shortcut or NULL
    formatting         TEXT NOT NULL,            -- JSON { capitalisation, punctuation, bullets, … }
    created_at         INTEGER NOT NULL,
    updated_at         INTEGER NOT NULL
);
CREATE INDEX idx_modes_sort ON modes(sort_order);

-- History
CREATE TABLE history (
    id                TEXT PRIMARY KEY,          -- ulid
    created_at        INTEGER NOT NULL,
    mode_id           TEXT REFERENCES modes(id) ON DELETE SET NULL,
    mode_name_snapshot TEXT NOT NULL,            -- denormalised, survives mode deletion
    asr_model_id      TEXT NOT NULL,
    asr_model_version TEXT NOT NULL,
    audio_path        TEXT,                      -- relative to recordings dir, NULL if not retained
    audio_duration_ms INTEGER NOT NULL,
    raw_text          TEXT NOT NULL,
    processed_text    TEXT NOT NULL,
    language          TEXT,
    processing_ms     INTEGER NOT NULL,
    status            TEXT NOT NULL,             -- 'success' | 'error' | 'cancelled'
    error_message     TEXT,
    app_exe           TEXT,
    app_title         TEXT,
    context_payload   TEXT,                      -- JSON (inspectable)
    word_timings      TEXT                       -- JSON [{ text, start_ms, end_ms }] or NULL
);
CREATE INDEX idx_history_created ON history(created_at DESC);
CREATE INDEX idx_history_mode    ON history(mode_id);
CREATE INDEX idx_history_app     ON history(app_exe);

-- Vocabulary
CREATE TABLE vocabulary (
    id          TEXT PRIMARY KEY,
    word        TEXT NOT NULL,
    hint        TEXT,                            -- pronunciation or replacement hint
    weight      REAL NOT NULL DEFAULT 1.0,
    enabled     INTEGER NOT NULL DEFAULT 1,
    created_at  INTEGER NOT NULL
);
CREATE UNIQUE INDEX idx_vocab_word ON vocabulary(word);

-- Text replacements (applied post-ASR, pre-LLM)
CREATE TABLE replacements (
    id          TEXT PRIMARY KEY,
    pattern     TEXT NOT NULL,
    replacement TEXT NOT NULL,
    is_regex    INTEGER NOT NULL DEFAULT 0,
    case_sens   INTEGER NOT NULL DEFAULT 0,
    enabled     INTEGER NOT NULL DEFAULT 1,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL
);

-- Installed models registry (cache over filesystem)
CREATE TABLE models (
    id             TEXT PRIMARY KEY,             -- 'parakeet-en-v1', …
    kind           TEXT NOT NULL,                -- 'asr' | 'llm'
    display_name   TEXT NOT NULL,
    version        TEXT NOT NULL,
    size_bytes     INTEGER NOT NULL,
    path           TEXT NOT NULL,                -- relative to models dir
    backend        TEXT NOT NULL,                -- 'onnx' | 'py-sidecar' | 'llama-cpp'
    languages      TEXT NOT NULL,                -- JSON array
    installed_at   INTEGER NOT NULL,
    last_used_at   INTEGER
);

-- Analytics rollups — one row per UTC day per metric
CREATE TABLE analytics_daily (
    day_utc       TEXT NOT NULL,                 -- YYYY-MM-DD
    metric        TEXT NOT NULL,                 -- 'words' | 'seconds_spoken' | 'dictations' | 'seconds_saved' | 'corrections'
    mode_id       TEXT,                          -- NULL => across all modes
    value         REAL NOT NULL,
    PRIMARY KEY (day_utc, metric, COALESCE(mode_id, ''))
);
CREATE INDEX idx_daily_day ON analytics_daily(day_utc);

-- File transcription queue
CREATE TABLE file_jobs (
    id           TEXT PRIMARY KEY,
    source_path  TEXT NOT NULL,
    mode_id      TEXT,
    model_id     TEXT,
    status       TEXT NOT NULL,                  -- 'queued'|'running'|'done'|'error'|'cancelled'
    progress     REAL NOT NULL DEFAULT 0.0,
    result_path  TEXT,
    error_message TEXT,
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL
);

-- Per-dictation corrections (for analytics correction-rate metric)
CREATE TABLE corrections (
    history_id   TEXT PRIMARY KEY REFERENCES history(id) ON DELETE CASCADE,
    edit_distance INTEGER NOT NULL,
    observed_at  INTEGER NOT NULL
);
```

## Seed data

- Six built-in modes: `voice-to-text`, `message`, `email`, `note`, `meeting`, `super`.
- No seeded vocabulary or replacements.
- No seeded history.

## Migration policy

- **Forward-only.** A migration may add columns and tables. Destructive changes require a new major schema version and a user-visible warning.
- **Backups.** Before running migrations, Utterbar copies the DB to `orliq.db.backup-<ts>` and keeps the last 3.
