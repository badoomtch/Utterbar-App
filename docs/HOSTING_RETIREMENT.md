# Legacy Website Retirement

Decision recorded: 18 October 2026. GitHub is the public home for Utterbar.
Downloads and updates use GitHub Releases. Support goes to `jay@jayparry.me`.
The Cloudflare migration was cancelled before any preview or production
deployment. No Cloudflare hosting or database resource was created for it.

## What Has Not Been Deleted

The legacy site, DNS and Cloudways database remain in place. Website source
is retained in `Utterbar Website/`. No domain, email service, shared subscription,
database or server has been cancelled by the repository-home update.

The read-only production audit found historical data, including 18 contact
messages and old licence, activation, release, subscriber and webhook records.
These are not all represented by current Laravel routes. Do not assume that
deleting the source website directory preserves or backs up those records.

## Before Cancelling Hosting

1. Take an application-scoped database and files backup and verify it can be
   restored. Preserve personal records privately, not in this public repository.
2. Decide how long contact and historical records must be retained. Export
   anything needed before deleting the application or its database.
3. Confirm the new support inbox receives mail. Changing a link to an email
   address does not verify mailbox delivery. No test mail has been sent here.
4. Consider a domain redirect to the repository while old links remain in
   search results and installed app versions. Retiring hosting and retiring
   a domain are separate operations; an expired domain can be registered by
   someone else.
5. Preserve unrelated mail, verification and legacy-download DNS records.
   Do not assume all subdomains serve only the marketing site.
6. Check every dependant before cancelling a shared service. Cloudways server
   972654 also hosts other applications; this task does not authorise changes
   to them. Mail credentials and sender verification may also be shared.

The next desktop build removes the active About/changelog website links.
Existing installers are unchanged until rebuilt and published separately.
