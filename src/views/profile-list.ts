import type { Profile, Usage } from "../types";

export function esc(s: string): string {
  const d = document.createElement("div");
  d.textContent = s;
  return d.innerHTML;
}

export function renderProfileList(root: HTMLElement, profiles: Profile[]): void {
  root.innerHTML = `
    <ul class="profile-list">
      ${profiles.map(renderRow).join("")}
    </ul>
  `;
}

export function renderUsage(root: HTMLElement, profileId: string, usage: Usage): void {
  const el = root.querySelector<HTMLElement>(`[data-usage-for="${CSS.escape(profileId)}"]`);
  if (!el) return;
  switch (usage.status) {
    case "ok":
      el.innerHTML = [
        usage.fiveHour ? usageBar("5h", usage.fiveHour.utilization, usage.fiveHour.resetsAt) : "",
        usage.sevenDay ? usageBar("7d", usage.sevenDay.utilization, usage.sevenDay.resetsAt) : "",
      ].join("");
      break;
    case "no-credentials":
      el.innerHTML = `<span class="usage-note" title="Log into Claude Code with this profile once to see usage">no usage data</span>`;
      break;
    case "token-expired":
      el.innerHTML = `<span class="usage-note" title="${esc(usage.message ?? "")}">usage stale</span>`;
      break;
    case "error":
      el.innerHTML = `<span class="usage-note" title="${esc(usage.message ?? "")}">usage unavailable</span>`;
      break;
  }
}

function usageBar(label: string, pct: number, resetsAt: string | null): string {
  const clamped = Math.max(0, Math.min(100, pct));
  const level = clamped >= 90 ? " usage-high" : clamped >= 70 ? " usage-warn" : "";
  const title = resetsAt
    ? `${label} window: ${clamped.toFixed(0)}% used, resets ${new Date(resetsAt).toLocaleString()}`
    : `${label} window: ${clamped.toFixed(0)}% used`;
  return `
    <span class="usage-item${level}" title="${esc(title)}">
      <span class="usage-label">${label}</span>
      <span class="usage-track"><span class="usage-fill" style="width:${clamped}%"></span></span>
      <span class="usage-pct">${clamped.toFixed(0)}%</span>
    </span>
  `;
}

function renderRow(p: Profile): string {
  const codexEnabled = p.codex?.enabled ?? false;
  // Enabled tools are already visible as launch buttons; only "default"
  // carries extra information.
  const badges = p.isDefault ? `<span class="badge badge-muted">default</span>` : "";
  return `
    <li class="profile-row" data-id="${esc(p.id)}">
      <span class="dot" style="background:${esc(p.color)}"></span>
      <div class="profile-info">
        <span class="profile-name">${esc(p.name)}</span>
        <span class="profile-badges">${badges}</span>
      </div>
      <div class="usage" data-usage-for="${esc(p.id)}"></div>
      <div class="profile-actions">
        ${
          p.desktop.enabled
            ? `<button class="btn" data-action="launch-desktop" title="Launch Claude Desktop">Claude Desktop</button>`
            : ""
        }
        ${
          p.code.enabled
            ? `<button class="btn" data-action="launch-terminal" title="Claude Code command for this profile">Claude Code</button>`
            : ""
        }
        ${
          codexEnabled
            ? `<button class="btn" data-action="launch-codex" title="Codex command for this profile">Codex</button>`
            : ""
        }
        <button class="btn btn-icon" data-action="edit" title="Edit profile">✎</button>
        ${
          p.isDefault
            ? ""
            : `<button class="btn btn-icon btn-danger" data-action="delete" title="Delete profile">🗑</button>`
        }
      </div>
    </li>
  `;
}
