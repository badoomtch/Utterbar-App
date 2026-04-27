-- Tag history rows produced by Meeting Mode so the new /meetings route
-- can list them separately from regular dictations on /history. The
-- column is purely a discriminator; everything else (text, audio_path,
-- timestamps, speaker tags) already lives on the row from the v0.13.x
-- meeting mode work.
--
-- Default 0 so existing rows stay on the dictation side. The dictation
-- insert path checks the active mode's `is_meeting_mode` flag and sets
-- this column to 1 when true, 0 otherwise. Reprocess (history.rs) sets
-- it to 0 — reprocessed rows are derivative dictations, not meetings.
ALTER TABLE history ADD COLUMN is_meeting INTEGER NOT NULL DEFAULT 0;

-- Partial index over the (small) set of meeting rows. The /meetings
-- route filters on is_meeting=1 ORDER BY created_at DESC; with the
-- default index on created_at and a partial bias against the (huge)
-- zero bucket, the query stays cheap even at tens of thousands of
-- history rows. Same shape as the partial index on
-- pending_refinement from V0003.
CREATE INDEX idx_history_is_meeting
    ON history(is_meeting, created_at DESC)
    WHERE is_meeting = 1;
