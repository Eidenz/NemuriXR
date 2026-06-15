// Controller input: an OpenXR action set (aim/grip poses, trigger, squeeze,
// system button, haptics) plus the per-frame interaction logic — raycast the
// laser onto a set of target panels, click via trigger, grab-to-move via grip
// squeeze, and detect the SYSTEM double-press that toggles the menu.
//
// Generalised from monado-frame: the loop takes a list of `(TargetId, pose,
// size)` and returns where each panel was pointed at, what to move, and the
// laser ray to draw.
use std::time::Instant;

use anyhow::Result;
use openxr as xr;

use crate::mathx::{locate_pose, pose_compose, pose_invert, raycast};

const GRAB_START: f32 = 0.40; // grip FORCE to start grabbing a panel
const GRAB_RELEASE: f32 = 0.15;

/// Identifies an interactable panel for one frame (caller assigns the ids).
pub type TargetId = u32;

#[derive(Default)]
pub struct Interaction {
    /// Per-panel pointer hit in normalised (u, v, pressed) coordinates.
    pub pointers: Vec<(TargetId, (f32, f32, bool))>,
    /// The laser ray to draw this frame: (controller aim pose, length).
    pub laser: Option<(xr::Posef, f32)>,
    /// Panels a grab moved this frame; caller applies the new pose.
    pub moves: Vec<(TargetId, xr::Posef)>,
    /// True if a controller is interacting with any panel (for input blocking).
    pub pointing_panel: bool,
}

impl Interaction {
    pub fn pointer(&self, target: TargetId) -> Option<(f32, f32, bool)> {
        self.pointers.iter().find(|(t, _)| *t == target).map(|(_, p)| *p)
    }
}

pub struct Input {
    action_set: xr::ActionSet,
    left_path: xr::Path,
    right_path: xr::Path,
    select: xr::Action<f32>,
    grab: xr::Action<f32>,
    a_button: xr::Action<bool>,
    haptic: xr::Action<xr::Haptic>,
    aim_left: xr::Space,
    aim_right: xr::Space,
    grip_left: xr::Space,
    // A-button double-press detection state
    btn_prev: bool,
    last_press: Option<Instant>,
    btn_active_prev: bool,
    last_active_change: Option<Instant>,
    // grab state: (target, hand index, controller→panel relative pose)
    grabbed: Option<(TargetId, usize, xr::Posef)>,
    prev_click: [bool; 2],
}

impl Input {
    pub fn new(instance: &xr::Instance, session: &xr::Session<xr::Vulkan>) -> Result<Self> {
        let action_set = instance.create_action_set("nemurixr", "NemuriXR controls", 0)?;
        let left_path = instance.string_to_path("/user/hand/left")?;
        let right_path = instance.string_to_path("/user/hand/right")?;
        let aim = action_set.create_action::<xr::Posef>("aim", "Aim pose", &[left_path, right_path])?;
        let grip = action_set.create_action::<xr::Posef>("grippose", "Grip pose", &[left_path, right_path])?;
        let select = action_set.create_action::<f32>("select", "Select", &[left_path, right_path])?;
        let grab = action_set.create_action::<f32>("grab", "Grab", &[left_path, right_path])?;
        let a_button = action_set.create_action::<bool>("abutton", "A button (show/hide)", &[right_path])?;
        let haptic = action_set.create_action::<xr::Haptic>("haptic", "Haptic tick", &[left_path, right_path])?;
        let index_profile = instance.string_to_path("/interaction_profiles/valve/index_controller")?;
        instance.suggest_interaction_profile_bindings(
            index_profile,
            &[
                xr::Binding::new(&aim, instance.string_to_path("/user/hand/left/input/aim/pose")?),
                xr::Binding::new(&aim, instance.string_to_path("/user/hand/right/input/aim/pose")?),
                xr::Binding::new(&grip, instance.string_to_path("/user/hand/left/input/grip/pose")?),
                xr::Binding::new(&grip, instance.string_to_path("/user/hand/right/input/grip/pose")?),
                xr::Binding::new(&select, instance.string_to_path("/user/hand/left/input/trigger/value")?),
                xr::Binding::new(&select, instance.string_to_path("/user/hand/right/input/trigger/value")?),
                xr::Binding::new(&grab, instance.string_to_path("/user/hand/left/input/squeeze/force")?),
                xr::Binding::new(&grab, instance.string_to_path("/user/hand/right/input/squeeze/force")?),
                xr::Binding::new(&a_button, instance.string_to_path("/user/hand/right/input/a/click")?),
                xr::Binding::new(&haptic, instance.string_to_path("/user/hand/left/output/haptic")?),
                xr::Binding::new(&haptic, instance.string_to_path("/user/hand/right/output/haptic")?),
            ],
        )?;
        session.attach_action_sets(&[&action_set])?;
        let aim_left = aim.create_space(session, left_path, xr::Posef::IDENTITY)?;
        let aim_right = aim.create_space(session, right_path, xr::Posef::IDENTITY)?;
        let grip_left = grip.create_space(session, left_path, xr::Posef::IDENTITY)?;

        Ok(Self {
            action_set,
            left_path,
            right_path,
            select,
            grab,
            a_button,
            haptic,
            aim_left,
            aim_right,
            grip_left,
            btn_prev: false,
            last_press: None,
            btn_active_prev: false,
            last_active_change: None,
            grabbed: None,
            prev_click: [false, false],
        })
    }

    pub fn sync(&self, session: &xr::Session<xr::Vulkan>) -> Result<()> {
        session.sync_actions(&[(&self.action_set).into()])?;
        Ok(())
    }

    pub fn clear_grab(&mut self) {
        self.grabbed = None;
    }

    /// Left grip pose in `space` (for hand-locked panels, e.g. a wrist card).
    pub fn grip_left_pose(&self, space: &xr::Space, time: xr::Time) -> Option<xr::Posef> {
        locate_pose(&self.grip_left, space, time)
    }

    /// True on the frame a double-tap of the right controller's A button
    /// completes. Ignores edges briefly around an action active-state flip (a
    /// controller waking otherwise fakes a press).
    pub fn a_double_press(&mut self, session: &xr::Session<xr::Vulkan>) -> Result<bool> {
        let s = self.a_button.state(session, self.right_path)?;
        let active = s.is_active;
        let down = s.is_active && s.current_state;
        if active != self.btn_active_prev {
            self.btn_active_prev = active;
            self.last_active_change = Some(Instant::now());
        }
        let settled = self.last_active_change.is_none_or(|t| t.elapsed().as_millis() > 150);
        let mut toggled = false;
        if down && !self.btn_prev && settled {
            let now = Instant::now();
            if self.last_press.is_some_and(|t| now.duration_since(t).as_millis() < 400) {
                toggled = true;
                self.last_press = None;
            } else {
                self.last_press = Some(now);
            }
        }
        self.btn_prev = down;
        Ok(toggled)
    }

    /// A short haptic tick on the given hand (ignored if the controller is absent).
    pub fn pulse(&self, session: &xr::Session<xr::Vulkan>, hand_left: bool) {
        let path = if hand_left { self.left_path } else { self.right_path };
        let v = xr::HapticVibration::new().amplitude(0.4).frequency(0.0).duration(xr::Duration::from_nanos(25_000_000));
        let _ = self.haptic.apply_feedback(session, path, &v);
    }

    /// One frame of pointing/grabbing against `targets` (each `(id, pose, size)`).
    pub fn process(
        &mut self,
        session: &xr::Session<xr::Vulkan>,
        space: &xr::Space,
        time: xr::Time,
        targets: &[(TargetId, xr::Posef, (f32, f32))],
    ) -> Result<Interaction> {
        let mut out = Interaction::default();
        let hands = [(self.left_path, &self.aim_left), (self.right_path, &self.aim_right)];

        // Continue an active grab (drop it if its panel went away or grip released).
        if let Some((tgt, hand, rel)) = self.grabbed {
            if !targets.iter().any(|(t, _, _)| *t == tgt) {
                self.grabbed = None;
            } else {
                let (path, aim) = hands[hand];
                let held = self.grab.state(session, path)?.current_state > GRAB_RELEASE;
                match locate_pose(aim, space, time) {
                    Some(pose) if held => {
                        out.pointing_panel = true;
                        out.moves.push((tgt, pose_compose(&pose, &rel)));
                    }
                    _ => self.grabbed = None,
                }
            }
        }

        if self.grabbed.is_none() && !targets.is_empty() {
            for (i, (path, aim)) in hands.iter().enumerate() {
                let Some(pose) = locate_pose(aim, space, time) else {
                    self.prev_click[i] = false;
                    continue;
                };
                // Nearest target the ray hits.
                let mut best: Option<(TargetId, f32, (f32, f32))> = None;
                for (tgt, ppose, psize) in targets {
                    if let Some((u, v, t)) = raycast(&pose, ppose, *psize) {
                        if best.is_none_or(|(_, bt, _)| t < bt) {
                            best = Some((*tgt, t, (u, v)));
                        }
                    }
                }
                if let Some((tgt, t, (u, v))) = best {
                    out.laser = Some((pose, t));
                    out.pointing_panel = true;
                    let down = self.select.state(session, *path)?.current_state > 0.5;
                    if down && !self.prev_click[i] {
                        self.pulse(session, *path == self.left_path);
                    }
                    self.prev_click[i] = down;
                    let grip = self.grab.state(session, *path)?.current_state;
                    if self.grabbed.is_none() && grip > GRAB_START {
                        let ppose = targets.iter().find(|(t, _, _)| *t == tgt).map(|(_, p, _)| *p).unwrap();
                        self.grabbed = Some((tgt, i, pose_compose(&pose_invert(&pose), &ppose)));
                        out.laser = None;
                    } else {
                        out.pointers.push((tgt, (u, v, down)));
                    }
                } else {
                    self.prev_click[i] = false;
                }
            }
        }

        Ok(out)
    }
}
