import { api, onProfilesChanged } from "./api";
import type { Profile } from "./types";
import { isAppError } from "./types";
import { renderProfileList } from "./views/profile-list";
import { showCreateDialog, showDeleteDialog, showEditDialog } from "./views/dialogs";

let profiles: Profile[] = [];
let platform = "macos";

const listRoot = () => document.querySelector<HTMLElement>("#profile-list-root")!;
const toastRoot = () => document.querySelector<HTMLElement>("#toast-root")!;

function toast(message: string, isError = false): void {
  const el = document.createElement("div");
  el.className = `toast${isError ? " toast-error" : ""}`;
  el.textContent = message;
  toastRoot().appendChild(el);
  setTimeout(() => el.remove(), isError ? 6000 : 2500);
}

function reportError(e: unknown): void {
  if (isAppError(e)) {
    toast(e.message, true);
  } else {
    toast(String(e), true);
  }
}

async function refresh(): Promise<void> {
  profiles = await api.listProfiles();
  renderProfileList(listRoot(), profiles);
}

function profileFromRow(el: Element): Profile | undefined {
  const id = el.closest<HTMLElement>(".profile-row")?.dataset.id;
  return profiles.find((p) => p.id === id);
}

async function handleAction(action: string, profile: Profile): Promise<void> {
  switch (action) {
    case "launch-desktop": {
      const result = await api.launchDesktop(profile.id);
      toast(
        result === "focusedExisting"
          ? `${profile.name} is already running — brought to front`
          : `Launching Claude Desktop (${profile.name})`,
      );
      break;
    }
    case "launch-terminal":
      await api.launchTerminal(profile.id);
      toast(`Opening terminal (${profile.name})`);
      break;
    case "edit":
      showEditDialog(profile, async (patch) => {
        try {
          await api.updateProfile(profile.id, patch);
          await refresh();
        } catch (e) {
          reportError(e);
        }
      });
      break;
    case "delete":
      showDeleteDialog(profile, platform, async (opts) => {
        try {
          const report = await api.deleteProfile(profile.id, opts);
          for (const w of report.warnings) toast(w, true);
          toast(`Deleted ${profile.name}`);
          await refresh();
        } catch (e) {
          reportError(e);
        }
      });
      break;
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  document.querySelector("#app")!.innerHTML = `
    <header class="app-header">
      <h1>Claude Account Switcher</h1>
      <button class="btn btn-primary" id="add-profile">+ New profile</button>
    </header>
    <main id="profile-list-root"></main>
    <footer class="app-footer" id="env-info"></footer>
    <div id="toast-root"></div>
  `;

  document.querySelector("#add-profile")!.addEventListener("click", () => {
    showCreateDialog(async (input) => {
      try {
        await api.createProfile(input);
        await refresh();
      } catch (e) {
        reportError(e);
      }
    });
  });

  listRoot().addEventListener("click", (e) => {
    const button = (e.target as HTMLElement).closest<HTMLElement>("[data-action]");
    if (!button) return;
    const profile = profileFromRow(button);
    if (!profile) return;
    handleAction(button.dataset.action!, profile).catch(reportError);
  });

  onProfilesChanged((updated) => {
    profiles = updated;
    renderProfileList(listRoot(), profiles);
  });

  try {
    const env = await api.detectEnvironment();
    platform = env.platform;
    const parts: string[] = [];
    parts.push(
      env.desktopAppPath ? "Claude Desktop: found" : "Claude Desktop: not found",
    );
    parts.push(env.claudeCodeOnPath ? "Claude Code: on PATH" : "Claude Code: not on PATH");
    document.querySelector("#env-info")!.textContent = parts.join("  ·  ");
    await refresh();
  } catch (e) {
    reportError(e);
  }
});
