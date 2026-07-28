export interface DesktopConfig {
  enabled: boolean;
  userDataDir: string;
}

export interface CodeConfig {
  enabled: boolean;
  configDir: string;
  shimName: string;
  shimInstalled: boolean;
}

export interface Profile {
  id: string;
  name: string;
  slug: string;
  color: string;
  isDefault: boolean;
  desktop: DesktopConfig;
  code: CodeConfig;
  createdAt: string;
  lastLaunchedAt: string | null;
}

export interface Settings {
  claudeDesktopPath: string | null;
  claudeCodePath: string | null;
  shimDir: string | null;
  terminal: string;
  hideWindowOnClose: boolean;
}

export interface NewProfile {
  name: string;
  color: string | null;
  desktopEnabled: boolean;
  codeEnabled: boolean;
}

export interface ProfileUpdate {
  name?: string | null;
  color?: string | null;
  desktopEnabled?: boolean | null;
  codeEnabled?: boolean | null;
}

export interface DeleteOptions {
  deleteDesktopDir: boolean;
  deleteCodeDir: boolean;
  deleteKeychainEntry: boolean;
  removeShim: boolean;
}

export interface DeleteReport {
  desktopDirDeleted: boolean;
  codeDirDeleted: boolean;
  keychainEntryDeleted: boolean;
  shimRemoved: boolean;
  warnings: string[];
}

export interface ShimStatus {
  installed: boolean;
  path: string | null;
  onPath: boolean;
  drifted: boolean;
}

export interface EnvInfo {
  platform: string;
  desktopAppPath: string | null;
  claudeCodeOnPath: boolean;
}

export interface LaunchOutcome {
  kind: "launched" | "focusedExisting";
  firstLogin: boolean;
}

export interface WindowUsage {
  utilization: number;
  resetsAt: string | null;
}

export interface Usage {
  status: "ok" | "no-credentials" | "token-expired" | "error";
  fiveHour: WindowUsage | null;
  sevenDay: WindowUsage | null;
  message: string | null;
}

export interface AppError {
  kind: "not-found" | "validation" | "io" | "launch" | "internal";
  message: string;
  detail: string | null;
}

export function isAppError(e: unknown): e is AppError {
  return typeof e === "object" && e !== null && "kind" in e && "message" in e;
}
