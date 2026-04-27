-- History rows produced by the streaming ASR path get an extra column.
-- `pending_refinement` is 0/1 and means: this row's text came from the
-- live streaming session (low-latency, smaller-context Zipformer) and
-- has not yet been re-transcribed by the offline refinement worker.
-- The Phase 2 worker drains rows where `pending_refinement = 1`,
-- re-runs the buffered audio through the offline (more accurate)
-- model, and overwrites raw_text + processed_text + word_timings, then
-- flips the flag back to 0. Default 0 so existing offline-path rows
-- (and any historical rows) skip the refinement queue automatically.
ALTER TABLE history ADD COLUMN pending_refinement INTEGER NOT NULL DEFAULT 0;

-- Partial index: only the small set of rows actually waiting for
-- refinement. The worker's drain query filters on this column, and we
-- want it answered without scanning the whole history table even when
-- the user has tens of thousands of rows. SQLite supports partial
-- indexes via the WHERE clause; the index excludes the (huge) zero
-- bucket and only carries the (small) one bucket, so it stays tiny.
CREATE INDEX idx_history_pending_refinement
    ON history(pending_refinement)
    WHERE pending_refinement = 1;
