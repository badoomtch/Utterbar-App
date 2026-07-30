# Public Repository Checklist

## Before publishing the repository

- [ ] Complete the local Git history rewrite in an isolated mirror.
- [ ] Run a history-aware secret scan with redacted output and resolve every
      finding.
- [ ] Verify the rewritten repository still builds and tests locally.
- [ ] Confirm GitHub Release delivery works for stable and beta channels.
- [ ] Ask Jay for explicit approval before force-pushing rewritten history.
- [ ] Make the GitHub repository public only after the rewritten history is
      visible and verified.

## Ongoing rules

- Keep credentials outside the repository.
- Use GitHub Releases for installers and release notes.
- Mark beta releases as GitHub prereleases and stable releases as ordinary
  published releases.
