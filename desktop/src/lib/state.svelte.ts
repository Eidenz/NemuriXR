// Shared reactive state (Svelte 5 runes). Polls the overlay/core over the Tauri
// IPC bridge; edits flow back via `save()` (debounced).
import { getConfig, setConfig, getState, setSleep } from "./api";
import type { Config, State } from "./types";

export const app = $state<{ config: Config | null; state: State | null; connected: boolean }>({
  config: null,
  state: null,
  connected: false,
});

let timer: ReturnType<typeof setInterval> | undefined;
let saveTimer: ReturnType<typeof setTimeout> | undefined;

export async function load() {
  try {
    app.config = await getConfig();
    app.connected = true;
  } catch (e) {
    console.error("getConfig failed", e);
    app.connected = false;
  }
}

/** Persist the current config now. */
export async function save() {
  if (!app.config) return;
  try {
    await setConfig($state.snapshot(app.config) as Config);
  } catch (e) {
    console.error("setConfig failed", e);
  }
}

/** Persist after a short idle (coalesces slider drags). */
export function saveSoon() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(save, 300);
}

export async function toggleSleep() {
  const next = !(app.state?.sleep_active ?? false);
  try {
    await setSleep(next);
    if (app.state) app.state.sleep_active = next;
  } catch (e) {
    console.error("setSleep failed", e);
  }
}

async function tick() {
  try {
    app.state = await getState();
    app.connected = true;
    if (!app.config) app.config = await getConfig();
  } catch {
    app.connected = false;
  }
}

export function startPolling() {
  stopPolling();
  load();
  tick();
  timer = setInterval(tick, 1000);
}

export function stopPolling() {
  if (timer) clearInterval(timer);
  timer = undefined;
}
