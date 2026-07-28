import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  CliTool,
  DeleteOptions,
  DeleteReport,
  EnvInfo,
  LaunchOutcome,
  NewProfile,
  Profile,
  ProfileUpdate,
  Settings,
  ShimStatus,
  Usage,
} from "./types";

export const api = {
  listProfiles: () => invoke<Profile[]>("list_profiles"),
  createProfile: (input: NewProfile) => invoke<Profile>("create_profile", { input }),
  updateProfile: (id: string, patch: ProfileUpdate) =>
    invoke<Profile>("update_profile", { id, patch }),
  deleteProfile: (id: string, opts: DeleteOptions) =>
    invoke<DeleteReport>("delete_profile", { id, opts }),
  launchDesktop: (id: string) => invoke<LaunchOutcome>("launch_desktop", { id }),
  launchTerminal: (id: string, tool: CliTool, terminal?: string) =>
    invoke<void>("launch_terminal", { id, tool, terminal: terminal ?? null }),
  terminalCommand: (id: string, tool: CliTool) =>
    invoke<string>("terminal_command", { id, tool }),
  getUsage: (id: string) => invoke<Usage>("get_usage", { id }),
  installShim: (id: string, tool: CliTool) =>
    invoke<ShimStatus>("install_shim", { id, tool }),
  removeShim: (id: string, tool: CliTool) => invoke<boolean>("remove_shim", { id, tool }),
  shimStatus: (id: string, tool: CliTool) =>
    invoke<ShimStatus>("shim_status", { id, tool }),
  detectEnvironment: () => invoke<EnvInfo>("detect_environment"),
  getSettings: () => invoke<Settings>("get_settings"),
  setSettings: (settings: Settings) => invoke<void>("set_settings", { settings }),
  revealPath: (path: string) => invoke<void>("reveal_path", { path }),
};

export function onProfilesChanged(handler: (profiles: Profile[]) => void) {
  return listen<Profile[]>("profiles-changed", (e) => handler(e.payload));
}
