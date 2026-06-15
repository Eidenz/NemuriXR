// Mirror of nemurixr-core's serde model (snake_case, enum tags as in Rust).

export type ListMode = "whitelist" | "blacklist";
export type VrcStatus = "join_me" | "active" | "ask_me" | "busy";
export type SleepPhase = "awake" | "prepare" | "sleep";
export type Sensitivity = "low" | "medium" | "high";

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
  detection_enabled: boolean;
  detection_always: boolean;
  detect_start: string; // "HH:MM"
  detect_end: string; // "HH:MM"
  detection_sensitivity: Sensitivity;
  detection_minutes: number;
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
  auto_launch_overlay: boolean;
  sleep: SleepConfig;
  brightness: BrightnessConfig;
  vrchat: VrchatConfig;
  osc: OscConfig;
}

export type LoginOutcome =
  | { kind: "logged_in"; username: string }
  | { kind: "needs_2fa"; methods: string[] }
  | { kind: "failed"; message: string };

export interface LoginStatus {
  logged_in: boolean;
  username: string | null;
}

export interface Friend {
  id: string;
  display_name: string;
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
