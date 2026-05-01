You're in.

Thanks for buying Utterbar Pro. Here's your licence.

Licence key:
{{ $displayKey }}

Open Utterbar -> Settings -> Licence, paste the key, and you're done.
Good on up to {{ $maxDevices }} devices.

One-click paste (if Utterbar is installed):
utterbar://licence/{{ $deepLinkKey }}
@if (! empty($isVolume))

For your IT team
----------------
This is a volume licence covering {{ $maxDevices }} devices. Each device
activates by pasting the key once, the same as a single-user purchase.

If you'd rather deploy the licence across a fleet without each user
pasting the key, an admin can sign in at https://utterbar.com/admin,
open this licence, and download a signed licence.bin. Drop the file at
%APPDATA%\Utterbar\licence.bin on each managed machine and Utterbar
treats it as the active Pro licence at startup, no activation prompt.
Full IT deployment doc: https://utterbar.com/docs/it-deployment
@endif

Lost this email later? Visit https://utterbar.com/licence, enter your email
({{ $email }}), and we'll send a magic link to manage your licence and devices.

-- Jay
Building Utterbar
utterbar.com
