# Security

## Reporting

Please report vulnerabilities privately via
[GitHub's "Report a vulnerability"](../../security/advisories/new) rather than a
public issue. This is a small side project maintained in spare time — expect a
best-effort reply, not an SLA.

## Scope

The app's security-relevant job is credential *isolation*: each profile points
Claude Desktop, Claude Code, and Codex at its own config directory so their
logins stay separate. The app never writes, refreshes, or transmits credentials.
It reads Claude Code's login token read-only, for the sole purpose of querying
Anthropic's usage API.

Especially interesting: anything that leaks one profile's credentials into
another, or sends a token anywhere other than the vendor's own API.

## Builds

macOS builds are unsigned and unnotarized, so downloads are only as trustworthy
as the GitHub release they came from. Fixes ship in a new release; only the
latest is supported.
