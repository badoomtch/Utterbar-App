-- Add `is_meeting_mode` to the modes table. When set on the active mode,
-- start_recording overrides capture_source to Mixed for the duration of
-- that recording and stop_recording chunks the captured audio into
-- ~30s windows for transcription rather than handing the whole buffer
-- to the offline recogniser at once. Defaults to 0 so every existing
-- mode keeps its current behaviour after the upgrade.
ALTER TABLE modes ADD COLUMN is_meeting_mode INTEGER NOT NULL DEFAULT 0;
