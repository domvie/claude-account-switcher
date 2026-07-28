# Claude Account Switcher

A small desktop app for people with more than one Claude account. It manages named
profiles ("Personal", "Work", …) and launches **Claude Desktop** or **Claude Code**
under the selected profile — each with its own login, settings, sessions, MCP servers,
skills, and plugins.

> **Status:** early development (pre-release). macOS first; Windows next; Linux planned (beta).

## How it works

- **Claude Desktop** is an Electron app, so each profile gets its own
  `--user-data-dir`, which keeps login sessions fully separate.
  ⚠️ This is standard Electron behavior but **not officially supported by Anthropic**
  and could break with any Claude Desktop update.
- **Claude Code** supports the official `CLAUDE_CONFIG_DIR` environment variable.
  Each profile gets its own config dir; credentials are isolated per directory
  (on macOS, Claude Code namespaces its Keychain entry per config dir automatically).
  The app installs per-profile shell shims (e.g. `claude-work`) and can open a
  terminal pre-configured for a profile.

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

## License

[MIT](LICENSE)
