import { invoke } from "@tauri-apps/api/core";
import type { Config, State } from "./types";

export const getConfig = () => invoke<Config>("get_config");
export const setConfig = (config: Config) => invoke("set_config", { config });
export const getState = () => invoke<State>("get_state");
export const setSleep = (active: boolean) => invoke("set_sleep", { active });
export const testSound = (kind: "join" | "leave") => invoke("test_sound", { kind });
