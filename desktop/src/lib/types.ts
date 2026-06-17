// Mirror of nemurixr-core's serde model (snake_case, enum tags as in Rust).

export type ListMode = "whitelist" | "blacklist";
export type VrcStatus = "join_me" | "active" | "ask_me" | "busy";
export type SleepPhase = "awake" | "prepare" | "sleep";
export type Sensitivity = "low" | "medium" | "high";
export interface SleepPose {
  name: string;
  gravity: [number, number, number];
}

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
export interface AudioLevel {
  set_volume: boolean;
  volume: number; // 0-100
  set_mic: boolean;
  mic_muted: boolean;
}
export interface AudioConfig {
  enabled: boolean;
  on_wake: AudioLevel;
  on_prepare: AudioLevel;
  on_sleep: AudioLevel;
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
  detection_poses: SleepPose[]; // calibrated sleep poses
  detection_pose_tolerance: number; // degrees
  wake: WakeConfig;
}
export interface WakeConfig {
  enabled: boolean;
  alarm_enabled: boolean;
  alarm_sound: string;
}
export interface CommandsConfig {
  enabled: boolean;
  on_wake: string;
  on_prepare: string;
  on_sleep: string;
}
export interface AutoAcceptConfig {
  enabled: boolean;
  list_mode: ListMode;
  player_ids: string[];
  only_when_sleep: boolean;
  max_players_enabled: boolean;
  max_players: number;
  invite_message_enabled: boolean;
  invite_message_slot: number;
  decline_message_enabled: boolean;
  decline_message_slot: number;
}
export interface JoinNotifyConfig {
  enabled: boolean;
  join_sound: string;
  leave_sound: string;
  only_when_alone: boolean;
  only_when_sleep: boolean;
  friends_only: boolean;
}
export interface StatusConfig {
  enabled: boolean;
  player_limit: number;
  below_status: VrcStatus;
  at_or_above_status: VrcStatus;
  only_when_sleep: boolean;
}
export interface SleepingPoseConfig {
  enabled: boolean;
  preset: string; // "gogo_loco" | "gorone" | "custom"
  lock_feet: boolean;
  on_back: OscMessage[];
  on_front: OscMessage[];
  on_left: OscMessage[];
  on_right: OscMessage[];
  foot_lock: OscMessage[];
  foot_unlock: OscMessage[];
}
export interface VrchatConfig {
  log_dir: string;
  auto_accept: AutoAcceptConfig;
  join_notifications: JoinNotifyConfig;
  status_automations: StatusConfig;
  sleeping_pose: SleepingPoseConfig;
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
export interface SafetyNetConfig {
  enabled: boolean;
  pose: boolean;
  pose_skip_if_trackers: boolean;
  pose_override_existing: boolean;
  mute_ingame: boolean;
  mute_device: boolean;
}
export interface Config {
  block_game_input: boolean;
  auto_launch_overlay: boolean;
  sleep: SleepConfig;
  brightness: BrightnessConfig;
  audio: AudioConfig;
  vrchat: VrchatConfig;
  osc: OscConfig;
  commands: CommandsConfig;
  safety_net: SafetyNetConfig;
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

export interface InviteMessage {
  slot: number;
  message: string;
  can_update: boolean;
  cooldown_minutes: number;
}

export interface State {
  sleep_phase: SleepPhase;
  player_count: number;
  vrchat_world: string | null;
  vrchat_status: string | null;
  brightness_backend: string | null;
  osc_target: string | null;
  audio_target: string | null;
  overlay_running: boolean;
}
