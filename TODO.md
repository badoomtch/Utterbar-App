# Utterbar Work Tracker

## Current priorities

1. Complete the public-repository scrub: verify rewritten history contains no
   secrets, replace the remote only after an explicit final approval, and make
   the repository public afterwards.
2. Verify GitHub Release delivery on both channels with a test release:
   ordinary release for stable and prerelease for beta.
3. Continue the app-language review beta. Native-speaker review is required
   before a language reaches a stable release.
4. Investigate and resolve reported audio-capture tail loss using reproducible
   recordings and logs.

## Product invariants

- Free, open-source, local-first desktop dictation.
- No payments, accounts, licences, telemetry, or cloud transcription.
- GitHub Releases provides desktop installers and update metadata.
- Stable and beta remain separate update channels.
