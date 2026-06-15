// Mirror of nemurixr-core's serde model (snake_case, enum tags as in Rust).

export type ListMode = "whitelist" | "blacklist";
export type VrcStatus = "join_me" | "active" | "ask_me" | "busy";
export type SleepPhase = "awake" | "prepare" | "sleep";

export interface BrightnessLevel {
  brightness: number; // 0-100
  fan: number; // 0-100 (Beyond only)
  transition_seconds: number; // fade time into this level
}
export interface BrightnessConfig {
  enabled: boolean;
  on_wake: BrightnessLevel;
  on_prepare: BrightnessLevel;
  on_sleep: BrightnessLevel;
}
export interface SleepConfig {
  schedule_enabled: boolean;
  sleep_at: string; // "HH:MM"
  wake_at: string;
}
export interface AutoAcceptConfig {
  enabled: boolean;
  list_mode: ListMode;
  player_ids: string[];
  only_when_sleep: boolean;
  max_players_enabled: boolean;
  max_players: number;
}
export interface JoinNotifyConfig {
  enabled: boolean;
  join_sound: string;
  leave_sound: string;
  only_when_alone: boolean;
  only_when_sleep: boolean;
}
export interface StatusConfig {
  enabled: boolean;
  player_limit: number;
  below_status: VrcStatus;
  at_or_above_status: VrcStatus;
  only_when_sleep: boolean;
}
export interface VrchatConfig {
  log_dir: string;
  auto_accept: AutoAcceptConfig;
  join_notifications: JoinNotifyConfig;
  status_automations: StatusConfig;
}
export type OscArg =
  | { kind: "bool"; value: boolean }
  | { kind: "int"; value: number }
  | { kind: "float"; value: number }
  | { kind: "string"; value: string };
export interface OscMessage {
  address: string;
  args: OscArg[];
  delay_ms: number;
}
export interface OscConfig {
  host: string;
  port: number;
  use_oscquery: boolean;
  on_wake: OscMessage[];
  on_prepare: OscMessage[];
  on_sleep: OscMessage[];
}
export interface Config {
  block_game_input: boolean;
  sleep: SleepConfig;
  brightness: BrightnessConfig;
  vrchat: VrchatConfig;
  osc: OscConfig;
}

export interface State {
  sleep_phase: SleepPhase;
  player_count: number;
  vrchat_world: string | null;
  vrchat_status: string | null;
  brightness_backend: string | null;
  osc_target: string | null;
  overlay_running: boolean;
}
