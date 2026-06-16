// Sleeping-pose OSC presets, mapping a lying direction to avatar OSC.
// Values sourced from OyasumiVR's sleeping-animation presets.
import type { OscMessage } from "./types";

const i = (address: string, value: number): OscMessage => ({ address, args: [{ kind: "int", value }], delay_ms: 0 });
const f = (address: string, value: number): OscMessage => ({ address, args: [{ kind: "float", value }], delay_ms: 0 });
const b = (address: string, value: boolean): OscMessage => ({ address, args: [{ kind: "bool", value }], delay_ms: 0 });

export interface PosePreset {
  id: string;
  label: string;
  lock_feet: boolean;
  on_back: OscMessage[];
  on_front: OscMessage[];
  on_left: OscMessage[];
  on_right: OscMessage[];
  foot_lock: OscMessage[];
  foot_unlock: OscMessage[];
}

export const POSE_PRESETS: PosePreset[] = [
  {
    id: "gogo_loco",
    label: "GoGo Loco",
    lock_feet: true,
    on_back: [
      i("/avatar/parameters/Go/VRCEmote", 237),
      f("/avatar/parameters/Go/Float", 0.04),
      f("/avatar/parameters/Go/PoseRadial", 0.5),
    ],
    // No dedicated face-down pose in GoGo; fall back to lying on back.
    on_front: [
      i("/avatar/parameters/Go/VRCEmote", 237),
      f("/avatar/parameters/Go/Float", 0.04),
      f("/avatar/parameters/Go/PoseRadial", 0.5),
    ],
    on_left: [
      i("/avatar/parameters/Go/VRCEmote", 243),
      f("/avatar/parameters/Go/Float", 0.03),
      b("/avatar/parameters/Go/Mirror", false),
      f("/avatar/parameters/Go/PoseRadial", 0.0),
    ],
    on_right: [
      i("/avatar/parameters/Go/VRCEmote", 243),
      f("/avatar/parameters/Go/Float", 0.03),
      b("/avatar/parameters/Go/Mirror", true),
      f("/avatar/parameters/Go/PoseRadial", 0.0),
    ],
    foot_lock: [b("/avatar/parameters/Go/Stationary", true)],
    foot_unlock: [b("/avatar/parameters/Go/Stationary", false)],
  },
  {
    id: "gorone",
    label: "ごろ寝システム EX (Sleep System)",
    lock_feet: true,
    on_back: [i("/avatar/parameters/VRCSupine", 1)],
    on_front: [i("/avatar/parameters/VRCSupine", 0)],
    on_left: [i("/avatar/parameters/VRCSupine", 3)],
    on_right: [i("/avatar/parameters/VRCSupine", 2)],
    foot_lock: [b("/avatar/parameters/VRCFootAnchor", true), b("/avatar/parameters/VRCLockPose", true)],
    foot_unlock: [b("/avatar/parameters/VRCFootAnchor", false), b("/avatar/parameters/VRCLockPose", false)],
  },
];
