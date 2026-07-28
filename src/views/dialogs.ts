import type { DeleteOptions, NewProfile, Profile, ProfileUpdate } from "../types";
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
      <menu>
        <button value="cancel" class="btn">Cancel</button>
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
      <label class="check"><input type="checkbox" name="code" ${profile.code.enabled ? "checked" : ""} /> Claude Code (terminal)</label>`
      }
      <menu>
        <button value="cancel" class="btn">Cancel</button>
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
    });
  });
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
        platform === "macos"
          ? `<label class="check"><input type="checkbox" name="keychain" /> Remove its Claude Code login from the Keychain</label>`
          : ""
      }
      <label class="check"><input type="checkbox" name="shim" checked /> Remove the <code>${esc(profile.code.shimName)}</code> shell command</label>
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
      deleteKeychainEntry: data.get("keychain") === "on",
      removeShim: data.get("shim") === "on",
    });
  });
}
