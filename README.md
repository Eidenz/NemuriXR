# NemuriXR

A **sleeping utility for VR on Linux/Monado**, inspired by
[OyasumiVR](https://github.com/Raphiiko/Oyasumi). *Nemuri* (眠り) means "sleep".

It comes in two parts that talk over a local socket:

- **Desktop app** (`desktop/`) — a **Tauri v2 + Svelte 5** window that is also the
  always-on **engine**: it owns the config + live state, runs the
  schedule/brightness/VRChat/OSC automations, and hosts the IPC server. It **lives
  in the system tray** — closing the window hides it (the engine keeps running);
  quit fully from the tray. Because it doesn't need VR, the VRChat features work in
  desktop VRChat too. Visual style is glassmorphism over a cinematic night
  backdrop.
- **Overlay** (`crates/overlay`) — an in-headset OpenXR overlay (`XR_EXTX_overlay`,
  egui on Vulkan quad layers, built on the [monado-frame](../monado-frame)
  foundation). It shows a compact **quick menu of toggles** (Sleep Mode + feature
  on/off switches) and is a thin **IPC client** of the desktop engine: it reflects
  live state and sends commands. If the desktop app isn't running it shows an
  offline notice.

The desktop is the engine + full setup; the overlay is a quick in-VR remote. They
share one config and live state through `nemurixr-core`.

## Status

**Foundation + scaffolding done.** The overlay renders its toggle menu and runs
the IPC server; the desktop app's shell, glass theme, Status screen, and the
**Brightness & Fans** screen are built and wired end-to-end over IPC. The
automation engines themselves are the next milestones.

### Roadmap

1. ✅ Overlay foundation + quick-menu toggles + manual Sleep Mode.
2. ✅ Workspace split: `core` (shared model + IPC); **desktop = engine + IPC
   server + tray**; overlay = thin IPC client. Glass desktop shell + Status +
   Brightness screens.
3. ✅ **VRChat join/leave (login-free)** — tails the VRChat log (VR *or* desktop
   mode) for live player count + current world, and plays join/leave
   notification sounds (only-when-alone / only-when-sleep). Auto-detects the
   Proton log dir; self-aware; grace period on world entry.
4. ✅ **Three sleep phases** — Awake → **Prepare** → Sleep, each with its own
   brightness/fan + OSC. A "Prepare to sleep" button sits in the overlay and the
   desktop Status screen.
5. ✅ **Brightness & Fans engine** — per-phase brightness/fan via Bigscreen Beyond
   HID ([bsb-control](../bsb-control) protocol) with a `libmonado` fallback;
   auto-detects the backend; **timed fades** into each phase (configurable, and
   cancelable when the phase changes mid-fade); "Preview on headset" buttons.
6. ✅ **OSC automations** — per-phase, sequenced OSC messages with per-message
   delays; VRChat's OSC port found via OSCQuery (mDNS), manual host/port fallback.
7. ✅ **Sleep schedule** — enters/leaves sleep mode at set times (edge-triggered,
   so a manual toggle still overrides until the next scheduled time).
8. ⏳ **VRChat API** — auto-accept invites + status automations (needs login).
9. ⏳ **Motion-based sleep detection** (a later version).

## Layout

```
nemurixr/                  (Rust workspace: overlay + core)
  crates/
    core/                  nemurixr-core — shared Config, live State, IPC protocol
    overlay/               nemurixr-overlay — OpenXR overlay, thin IPC client
  desktop/                 (its own workspace)
    src/                   Svelte 5 (SvelteKit, adapter-static SPA): glass UI
    src-tauri/             Tauri v2 backend — the engine + IPC server (+ tray)
```

The overlay's `src/overlay/` module is a trimmed lift of monado-frame's overlay
toolkit (session/panel/input/laser).

## Build & run

**Desktop app (the engine — run this first, keep it in the tray):**

```bash
cd desktop
pnpm install
pnpm tauri dev      # or: pnpm tauri build
```

Closing the window hides it to the tray; the engine keeps running. Quit fully from
the tray menu. (Linux tray needs a StatusNotifier host — native on KDE; on GNOME
install the AppIndicator extension and `libappindicator-gtk3`.)

**Overlay** (run with your Monado / Envision VR session active):

```bash
cargo build
cargo run -p nemurixr-overlay
```

Requires an OpenXR runtime (Monado) with `XR_KHR_vulkan_enable2` and
`XR_EXTX_overlay`. Double-press **SYSTEM** to open the quick menu; point + trigger
to click, grip to move it. It connects to the desktop engine; if the desktop app
isn't running it shows an offline notice.

## Configuration

Settings persist at `~/.config/nemurixr/config.json` (XDG-aware), owned by the
core and edited by either UI.

Overlay env vars: `NEMURI_OPACITY` (0–1, default 0.92), `NEMURI_NO_ALPHA`,
`NEMURI_NO_LASER`.

## Hardware control (milestone 3)

Bigscreen Beyond brightness/fan control uses its HID protocol (VID `0x35BD` /
PID `0x0101`), needing the same udev rule as bsb-control:

```
KERNEL=="hidraw*", SUBSYSTEM=="hidraw", ATTRS{idVendor}=="35bd", ATTRS{idProduct}=="0101", MODE="0660", GROUP="wheel"
```

## IPC

A Unix socket at `$XDG_RUNTIME_DIR/nemurixr.sock` carries newline-delimited JSON
(`GetConfig` / `SetConfig` / `GetState` / `SetSleep`). The **desktop** backend is
the server (the engine); the **overlay** is the client. The desktop's own frontend
talks to the engine in-process via Tauri commands.
