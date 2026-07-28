import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  DeleteOptions,
  DeleteReport,
  EnvInfo,
  LaunchResult,
  NewProfile,
  Profile,
  ProfileUpdate,
  Settings,
  ShimStatus,
} from "./types";

export const api = {
  listProfiles: () => invoke<Profile[]>("list_profiles"),
  createProfile: (input: NewProfile) => invoke<Profile>("create_profile", { input }),
  updateProfile: (id: string, patch: ProfileUpdate) =>
    invoke<Profile>("update_profile", { id, patch }),
  deleteProfile: (id: string, opts: DeleteOptions) =>
    invoke<DeleteReport>("delete_profile", { id, opts }),
  launchDesktop: (id: string) => invoke<LaunchResult>("launch_desktop", { id }),
  launchTerminal: (id: string) => invoke<void>("launch_terminal", { id }),
  installShim: (id: string) => invoke<ShimStatus>("install_shim", { id }),
  removeShim: (id: string) => invoke<boolean>("remove_shim", { id }),
  shimStatus: (id: string) => invoke<ShimStatus>("shim_status", { id }),
  detectEnvironment: () => invoke<EnvInfo>("detect_environment"),
  getSettings: () => invoke<Settings>("get_settings"),
  setSettings: (settings: Settings) => invoke<void>("set_settings", { settings }),
  revealPath: (path: string) => invoke<void>("reveal_path", { path }),
};

export function onProfilesChanged(handler: (profiles: Profile[]) => void) {
  return listen<Profile[]>("profiles-changed", (e) => handler(e.payload));
}
