# Claude Account Switcher

A small desktop app for people with more than one Claude account. It manages named
profiles ("Personal", "Work", …) and launches **Claude Desktop**, **Claude Code**, or
**OpenAI Codex CLI** under the selected profile — each with its own login, settings,
sessions, MCP servers, skills, and plugins.

> **Status:** early development. **macOS only** — the code compiles on Windows and
> Linux, but the platform layer there is still stubbed out, so launching does not
> work yet and no builds are published for those platforms. Contributions welcome.

![Claude Account Switcher](screenshot.png)

## Install

**macOS only for now.** Grab the `.dmg` (universal binary) from
[Releases](../../releases). It is **unsigned** (no paid developer certificate
yet), so Gatekeeper will object once: after moving the app to Applications,
either right-click → **Open**, or go to System Settings → Privacy & Security →
**"Open Anyway"** after the first blocked launch. Alternatively:

```sh
xattr -cr "/Applications/Claude Account Switcher.app"
```

Windows and Linux builds are not published yet — see the status note above.


## Known quirks

- **Email login codes land in the wrong window.** When you log a new profile in
  by email, the "Sign in" button in the verification email opens a `claude://`
  link — and the OS routes that to your *main* Claude instance, so the code
  shows up there. This can't be intercepted; just copy the code from wherever
  it appears into the login screen of the *new* window. The app shows a
  reminder on a profile's first login.
- **Usage stats ask for Keychain access (macOS).** The per-profile usage bars
  read each profile's Claude Code login token (read-only) to query Anthropic's
  usage API. On macOS that token lives in the Keychain, so the first read
  triggers an "allow access?" prompt — click **Always Allow** to stop it
  reappearing. The app never modifies or refreshes tokens.
- **Claude Code launches start in your home directory.** The recommended flow
  is the profile's **Terminal** dialog: copy the shown command and paste it in
  your own terminal, in the project directory you actually want to work in.

## Development

```sh
npm install
npm run tauri dev
```

Built with [Tauri v2](https://v2.tauri.app) (Rust + TypeScript).

## Disclaimer

This is an independent open-source project. It is **not affiliated with,
endorsed by, or supported by Anthropic or OpenAI**. "Claude" is a trademark of
Anthropic, PBC; "OpenAI" and "Codex" are trademarks of OpenAI. The app relies
partly on undocumented behavior (see Known quirks) that could change at any
time. It reads login tokens only to display usage and never sends them
anywhere except to the respective vendor's own API.

## License

[MIT](LICENSE)
