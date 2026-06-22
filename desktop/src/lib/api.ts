import { invoke } from "@tauri-apps/api/core";
import type { Config, Friend, InviteMessage, LoginOutcome, LoginStatus, SleepPhase, State } from "./types";

export const getConfig = () => invoke<Config>("get_config");
export const setConfig = (config: Config) => invoke("set_config", { config });
export const getState = () => invoke<State>("get_state");
export const setPhase = (phase: SleepPhase) => invoke("set_phase", { phase });
export const testSound = (kind: "join" | "leave") => invoke("test_sound", { kind });
export const testAlarm = () => invoke("test_alarm");
export const stopAlarm = () => invoke("stop_alarm");
export const applyBrightness = (which: SleepPhase) => invoke("apply_brightness", { which });
export const sendOsc = (which: SleepPhase) => invoke("send_osc", { which });
export const applyAudio = (which: SleepPhase) => invoke("apply_audio", { which });
export const testCommand = (which: SleepPhase) => invoke("test_command", { which });
export const testSleepingPose = (which: "back" | "front" | "left" | "right") =>
  invoke("test_sleeping_pose", { which });
export const launchOverlay = () => invoke<boolean>("launch_overlay");

// Version + updates
export interface UpdateInfo {
  version: string;
  url: string;
}
export const appVersion = () => invoke<string>("app_version");
export const checkUpdate = () => invoke<UpdateInfo | null>("check_update");

// Bigscreen Beyond udev rule
export type BeyondStatus = "absent" | "needs_rule" | "ready";
export const beyondStatus = () => invoke<BeyondStatus>("beyond_status");
export const beyondRuleText = () => invoke<string>("beyond_rule_text");
export const installBeyondRule = () => invoke("install_beyond_rule");

export type MessageKind = "message" | "requestResponse";

// VRChat account
export const vrchatStatus = () => invoke<LoginStatus>("vrchat_status");
export const vrchatLogin = (username: string, password: string) => invoke<LoginOutcome>("vrchat_login", { username, password });
export const vrchatVerify2fa = (method: string, code: string) => invoke<LoginOutcome>("vrchat_verify_2fa", { method, code });
export const vrchatLogout = () => invoke<LoginStatus>("vrchat_logout");
export const vrchatFriends = () => invoke<Friend[]>("vrchat_friends");
export const vrchatRefreshFriends = () => invoke<Friend[]>("vrchat_refresh_friends");
export const vrchatMessages = (kind: MessageKind) => invoke<InviteMessage[]>("vrchat_messages", { kind });
export const vrchatUpdateMessage = (kind: MessageKind, slot: number, text: string) =>
  invoke<InviteMessage[]>("vrchat_update_message", { kind, slot, text });
