import type { Profile } from "../types";

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

function renderRow(p: Profile): string {
  const badges = [
    p.desktop.enabled ? `<span class="badge">Desktop</span>` : "",
    p.code.enabled ? `<span class="badge">Code</span>` : "",
    p.isDefault ? `<span class="badge badge-muted">default</span>` : "",
  ].join("");
  return `
    <li class="profile-row" data-id="${esc(p.id)}">
      <span class="dot" style="background:${esc(p.color)}"></span>
      <div class="profile-info">
        <span class="profile-name">${esc(p.name)}</span>
        <span class="profile-badges">${badges}</span>
      </div>
      <div class="profile-actions">
        ${
          p.desktop.enabled
            ? `<button class="btn" data-action="launch-desktop" title="Launch Claude Desktop">Desktop</button>`
            : ""
        }
        ${
          p.code.enabled
            ? `<button class="btn" data-action="launch-terminal" title="Open a terminal with this profile">Terminal</button>`
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
