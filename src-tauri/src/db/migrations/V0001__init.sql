CREATE TABLE schema_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
INSERT INTO schema_meta(key, value) VALUES ('version', '1');

CREATE TABLE modes (
    id                 TEXT PRIMARY KEY,
    name               TEXT NOT NULL,
    icon               TEXT NOT NULL,
    is_builtin         INTEGER NOT NULL DEFAULT 0,
    is_favourite       INTEGER NOT NULL DEFAULT 0,
    sort_order         INTEGER NOT NULL DEFAULT 0,
    asr_model_id       TEXT,
    target_language    TEXT,
    post_processing    TEXT NOT NULL,
    context_rules      TEXT NOT NULL,
    auto_activation    TEXT NOT NULL,
    shortcut_override  TEXT,
    formatting         TEXT NOT NULL,
    created_at         INTEGER NOT NULL,
    updated_at         INTEGER NOT NULL
);
CREATE INDEX idx_modes_sort ON modes(sort_order);

CREATE TABLE history (
    id                 TEXT PRIMARY KEY,
    created_at         INTEGER NOT NULL,
    mode_id            TEXT REFERENCES modes(id) ON DELETE SET NULL,
    mode_name_snapshot TEXT NOT NULL,
    asr_model_id       TEXT NOT NULL,
    asr_model_version  TEXT NOT NULL,
    audio_path         TEXT,
    audio_duration_ms  INTEGER NOT NULL,
    raw_text           TEXT NOT NULL,
    processed_text     TEXT NOT NULL,
    language           TEXT,
    processing_ms      INTEGER NOT NULL,
    status             TEXT NOT NULL,
    error_message      TEXT,
    app_exe            TEXT,
    app_title          TEXT,
    context_payload    TEXT,
    word_timings       TEXT
);
CREATE INDEX idx_history_created ON history(created_at DESC);
CREATE INDEX idx_history_mode    ON history(mode_id);
CREATE INDEX idx_history_app     ON history(app_exe);

CREATE TABLE vocabulary (
    id         TEXT PRIMARY KEY,
    word       TEXT NOT NULL,
    hint       TEXT,
    weight     REAL NOT NULL DEFAULT 1.0,
    enabled    INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL
);
CREATE UNIQUE INDEX idx_vocab_word ON vocabulary(word);

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

CREATE TABLE models (
    id             TEXT PRIMARY KEY,
    kind           TEXT NOT NULL,
    display_name   TEXT NOT NULL,
    version        TEXT NOT NULL,
    size_bytes     INTEGER NOT NULL,
    path           TEXT NOT NULL,
    backend        TEXT NOT NULL,
    languages      TEXT NOT NULL,
    installed_at   INTEGER NOT NULL,
    last_used_at   INTEGER
);

CREATE TABLE analytics_daily (
    day_utc TEXT NOT NULL,
    metric  TEXT NOT NULL,
    -- Empty string represents "across all modes"; NULL is not used so this can participate
    -- in a composite PRIMARY KEY without COALESCE (SQLite rejects expressions in PK).
    mode_id TEXT NOT NULL DEFAULT '',
    value   REAL NOT NULL,
    PRIMARY KEY (day_utc, metric, mode_id)
);
CREATE INDEX idx_daily_day ON analytics_daily(day_utc);

CREATE TABLE file_jobs (
    id            TEXT PRIMARY KEY,
    source_path   TEXT NOT NULL,
    mode_id       TEXT,
    model_id      TEXT,
    status        TEXT NOT NULL,
    progress      REAL NOT NULL DEFAULT 0.0,
    result_path   TEXT,
    error_message TEXT,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL
);

CREATE TABLE corrections (
    history_id    TEXT PRIMARY KEY REFERENCES history(id) ON DELETE CASCADE,
    edit_distance INTEGER NOT NULL,
    observed_at   INTEGER NOT NULL
);
