import type {
  DeleteOptions,
  NewProfile,
  Profile,
  ProfileUpdate,
  ShimStatus,
} from "../types";
import { esc } from "./profile-list";

const COLORS = ["#D97757", "#6A9BCC", "#7C3AED", "#2E9E6B", "#C2483B", "#B58A2A"];

function openDialog(html: string): HTMLDialogElement {
  const dialog = document.createElement("dialog");
  dialog.innerHTML = html;
  document.body.appendChild(dialog);
  dialog.addEventListener("close", () => dialog.remove());
  dialog.showModal();
  return dialog;
}

export function showCreateDialog(onSubmit: (input: NewProfile) => void): void {
  const dialog = openDialog(`
    <form method="dialog" class="dialog-form">
      <h2>New profile</h2>
      <label>Name
        <input name="name" required placeholder="Work" autocomplete="off" />
      </label>
      <div class="color-row">
        ${COLORS.map(
          (c, i) =>
            `<label class="color-swatch"><input type="radio" name="color" value="${c}" ${i === 1 ? "checked" : ""} /><span style="background:${c}"></span></label>`,
        ).join("")}
      </div>
      <label class="check"><input type="checkbox" name="desktop" checked /> Claude Desktop</label>
      <label class="check"><input type="checkbox" name="code" checked /> Claude Code (terminal)</label>
      <label class="check"><input type="checkbox" name="codex" /> OpenAI Codex (terminal)</label>
      <menu>
        <button value="cancel" class="btn" formnovalidate>Cancel</button>
        <button value="ok" class="btn btn-primary">Create</button>
      </menu>
    </form>
  `);
  const form = dialog.querySelector("form")!;
  dialog.addEventListener("close", () => {
    if (dialog.returnValue !== "ok") return;
    const data = new FormData(form);
    onSubmit({
      name: String(data.get("name") ?? "").trim(),
      color: String(data.get("color") ?? COLORS[1]),
      desktopEnabled: data.get("desktop") === "on",
      codeEnabled: data.get("code") === "on",
      codexEnabled: data.get("codex") === "on",
    });
  });
}

export function showEditDialog(
  profile: Profile,
  onSubmit: (patch: ProfileUpdate) => void,
): void {
  const dialog = openDialog(`
    <form method="dialog" class="dialog-form">
      <h2>Edit ${esc(profile.name)}</h2>
      <label>Name
        <input name="name" required value="${esc(profile.name)}" autocomplete="off" />
      </label>
      <div class="color-row">
        ${COLORS.map(
          (c) =>
            `<label class="color-swatch"><input type="radio" name="color" value="${c}" ${c.toLowerCase() === profile.color.toLowerCase() ? "checked" : ""} /><span style="background:${c}"></span></label>`,
        ).join("")}
      </div>
      ${
        profile.isDefault
          ? `<p class="hint">This is the default profile — it uses your existing Claude data and can't be disabled or deleted.</p>`
          : `
      <label class="check"><input type="checkbox" name="desktop" ${profile.desktop.enabled ? "checked" : ""} /> Claude Desktop</label>
      <label class="check"><input type="checkbox" name="code" ${profile.code.enabled ? "checked" : ""} /> Claude Code (terminal)</label>
      <label class="check"><input type="checkbox" name="codex" ${profile.codex?.enabled ? "checked" : ""} /> OpenAI Codex (terminal)</label>`
      }
      <menu>
        <button value="cancel" class="btn" formnovalidate>Cancel</button>
        <button value="ok" class="btn btn-primary">Save</button>
      </menu>
    </form>
  `);
  const form = dialog.querySelector("form")!;
  dialog.addEventListener("close", () => {
    if (dialog.returnValue !== "ok") return;
    const data = new FormData(form);
    onSubmit({
      name: String(data.get("name") ?? "").trim(),
      color: String(data.get("color") ?? profile.color),
      desktopEnabled: profile.isDefault ? null : data.get("desktop") === "on",
      codeEnabled: profile.isDefault ? null : data.get("code") === "on",
      codexEnabled: profile.isDefault ? null : data.get("codex") === "on",
    });
  });
}

/// Shown when Claude Code is requested for a profile: the command to paste
/// into the user's own terminal, plus optional one-click launchers and
/// shim management.
export function showTerminalDialog(
  profile: Profile,
  toolLabel: string,
  shimName: string,
  command: string,
  shim: ShimStatus,
  onCopy: (command: string) => void,
  onOpen: (terminal: "Terminal" | "iTerm2") => void,
  onInstallShim: () => Promise<ShimStatus>,
): void {
  const shimSection = (s: ShimStatus): string => {
    if (profile.isDefault) return "";
    if (!s.installed) {
      return `<p class="hint">Tip: install a <code>${esc(shimName)}</code> command so you don't need the env var:
        <button type="button" class="btn" id="install-shim">Install shim</button></p>`;
    }
    const pathHint = s.onPath
      ? ""
      : `<br/>⚠ <code>${esc(s.path ?? "")}</code> is not on your PATH — add:
         <code class="selectable">export PATH="$HOME/.local/bin:$PATH"</code>`;
    return `<p class="hint">✓ <code>${esc(shimName)}</code> is installed${pathHint}</p>`;
  };

  const dialog = openDialog(`
    <form method="dialog" class="dialog-form">
      <h2>${esc(toolLabel)} — ${esc(profile.name)}</h2>
      <p class="hint">Run this in any terminal, from whatever directory you want to work in:</p>
      <div class="command-box"><code>${esc(command)}</code></div>
      <div id="shim-section">${shimSection(shim)}</div>
      <menu class="menu-left">
        <button value="copy" class="btn btn-primary">Copy command</button>
        <button value="open-terminal" class="btn">Open in Terminal</button>
        <button value="open-iterm" class="btn">Open in iTerm2</button>
      </menu>
    </form>
  `);

  dialog.querySelector("#install-shim")?.addEventListener("click", async () => {
    try {
      const status = await onInstallShim();
      dialog.querySelector("#shim-section")!.innerHTML = shimSection(status);
    } catch {
      // error is toasted by the caller
    }
  });

  dialog.addEventListener("close", () => {
    switch (dialog.returnValue) {
      case "copy":
        onCopy(command);
        break;
      case "open-terminal":
        onOpen("Terminal");
        break;
      case "open-iterm":
        onOpen("iTerm2");
        break;
    }
  });
}

/// First launch of a profile that has never logged in: explain the
/// email-verification deep-link quirk before the user hits it.
export function showFirstLoginHint(profile: Profile): void {
  openDialog(`
    <form method="dialog" class="dialog-form">
      <h2>First login for ${esc(profile.name)}</h2>
      <p class="hint">A new Claude window is opening — log in there with this profile's account.</p>
      <p class="hint"><strong>If you sign in by email:</strong> the "Sign in" button in the
      verification email opens your <em>main</em> Claude window and shows the code there.
      That's a macOS limitation (the link can only open one app). Just copy the code from
      wherever it appears and paste it into the <em>new</em> window's login screen.</p>
      <menu>
        <button value="ok" class="btn btn-primary">Got it</button>
      </menu>
    </form>
  `);
}

export function showDeleteDialog(
  profile: Profile,
  platform: string,
  onConfirm: (opts: DeleteOptions) => void,
): void {
  const dialog = openDialog(`
    <form method="dialog" class="dialog-form">
      <h2>Delete ${esc(profile.name)}?</h2>
      <p class="hint">The profile entry is always removed. Optionally also clean up its data — directories go to the Trash, not permanent deletion.</p>
      <label class="check"><input type="checkbox" name="desktopDir" /> Move Claude Desktop data to Trash<br/><code>${esc(profile.desktop.userDataDir)}</code></label>
      <label class="check"><input type="checkbox" name="codeDir" /> Move Claude Code config to Trash<br/><code>${esc(profile.code.configDir)}</code></label>
      ${
        profile.codex
          ? `<label class="check"><input type="checkbox" name="codexDir" /> Move Codex config to Trash<br/><code>${esc(profile.codex.configDir)}</code></label>`
          : ""
      }
      ${
        platform === "macos"
          ? `<label class="check"><input type="checkbox" name="keychain" /> Remove its Claude Code login from the Keychain</label>`
          : ""
      }
      <label class="check"><input type="checkbox" name="shim" checked /> Remove its shell commands (<code>${esc(profile.code.shimName)}</code>${profile.codex ? `, <code>${esc(profile.codex.shimName)}</code>` : ""})</label>
      <menu>
        <button value="cancel" class="btn">Cancel</button>
        <button value="ok" class="btn btn-danger-solid">Delete profile</button>
      </menu>
    </form>
  `);
  const form = dialog.querySelector("form")!;
  dialog.addEventListener("close", () => {
    if (dialog.returnValue !== "ok") return;
    const data = new FormData(form);
    onConfirm({
      deleteDesktopDir: data.get("desktopDir") === "on",
      deleteCodeDir: data.get("codeDir") === "on",
      deleteCodexDir: data.get("codexDir") === "on",
      deleteKeychainEntry: data.get("keychain") === "on",
      removeShim: data.get("shim") === "on",
    });
  });
}
