import { invoke } from "@tauri-apps/api/core";
import type { Config, LoginOutcome, LoginStatus, SleepPhase, State } from "./types";

export const getConfig = () => invoke<Config>("get_config");
export const setConfig = (config: Config) => invoke("set_config", { config });
export const getState = () => invoke<State>("get_state");
export const setPhase = (phase: SleepPhase) => invoke("set_phase", { phase });
export const testSound = (kind: "join" | "leave") => invoke("test_sound", { kind });
export const applyBrightness = (which: SleepPhase) => invoke("apply_brightness", { which });
export const sendOsc = (which: SleepPhase) => invoke("send_osc", { which });

// VRChat account
export const vrchatStatus = () => invoke<LoginStatus>("vrchat_status");
export const vrchatLogin = (username: string, password: string) => invoke<LoginOutcome>("vrchat_login", { username, password });
export const vrchatVerify2fa = (method: string, code: string) => invoke<LoginOutcome>("vrchat_verify_2fa", { method, code });
export const vrchatLogout = () => invoke<LoginStatus>("vrchat_logout");
