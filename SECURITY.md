# Security policy

## Supported versions

Only the latest tagged release is supported.

## Reporting

Please open a private GitHub security advisory instead of a public issue for
vulnerabilities involving native memory safety, command exposure or package
signing.

## Design notes

- The application does not inject into third-party processes.
- The native DLL is loaded only from application-controlled paths.
- No remote content is loaded in the WebView.
- No analytics or telemetry are included.
- Release binaries should be signed before public distribution.
