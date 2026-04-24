-- Add `english_variant` to the modes table so each mode can opt into BrE/AmE
-- spelling normalisation in post-processing. Defaults to `auto`, meaning
-- "leave whatever Parakeet transcribed alone" — the same behaviour every
-- existing mode has had since v0.3, so no mode changes meaning across this
-- upgrade. Stored as a short TEXT column rather than inside `formatting`
-- JSON because querying it per-dictation (hot path) benefits from a plain
-- column and it sits conceptually alongside `target_language`.
ALTER TABLE modes ADD COLUMN english_variant TEXT NOT NULL DEFAULT 'auto';
