# Windows notification integration

## What works in every build

The notification overlay, its design system, local feed, dismissal, clearing,
stacking and demo events work in normal MSI/NSIS builds.

The Rust command surface can later accept notifications from first-party
Moustache modules without changing the UI.

## Why arbitrary Windows notifications are different

Reading notifications sent by other applications uses
`Windows.UI.Notifications.Management.UserNotificationListener`. Windows
requires explicit user permission. The capability is tied to package identity,
so the reliable distribution route is MSIX.

Capturing a toast does not automatically suppress the original toast.
Globally disabling Windows notifications is a separate user/system setting and
is intentionally not changed silently.

## MSIX preparation included here

`packaging/msix/AppxManifest.xml` declares:

- `userNotificationListener`
- `runFullTrust`

`scripts/package-msix.ps1` prepares an MSIX layout from a release build. A real
release still needs a publisher identity and a trusted signing certificate.

## Production implementation checklist

1. Add a packaged WinRT notification adapter.
2. Request access from a visible UI action.
3. Store and display the permission state.
4. Subscribe to notification changes.
5. Convert app info, title, body and timestamp into `NotificationItem`.
6. Deduplicate by Windows notification ID.
7. Handle revoked permission.
8. Let the user choose whether to disable Windows banners in Settings.

This is intentionally capability-gated instead of pretending an unpackaged
process can always intercept and suppress every notification.
