// Shared reactive state (Svelte 5 runes). Polls the overlay/core over the Tauri
// IPC bridge; edits flow back via `save()` (debounced).
import { getConfig, setConfig, getState, setPhase, vrchatStatus, vrchatFriends } from "./api";
import type { Config, Friend, LoginStatus, SleepPhase, State } from "./types";

const ls = typeof localStorage !== "undefined" ? localStorage : null;
const NAMES_KEY = "nemurixr.friendNames";
function loadNames(): Record<string, string> {
  try {
    return JSON.parse(ls?.getItem(NAMES_KEY) ?? "{}");
  } catch {
    return {};
  }
}

export const app = $state<{
  config: Config | null;
  state: State | null;
  connected: boolean;
  vrchatLogin: LoginStatus;
  vrchatFriends: Friend[];
  /** id → display name cache, so whitelisted friends show names without a fetch. */
  friendNames: Record<string, string>;
}>({
  config: null,
  state: null,
  connected: false,
  vrchatLogin: { logged_in: false, username: null },
  vrchatFriends: [],
  friendNames: loadNames(),
});

function persistNames() {
  ls?.setItem(NAMES_KEY, JSON.stringify(app.friendNames));
}
/** Remember a friend's display name (persisted) for the whitelist chips. */
export function cacheFriendName(id: string, name: string) {
  app.friendNames[id] = name;
  persistNames();
}

let friendsLoaded = false;
/** Fetch the friends list once (cached across tab switches); `force` re-fetches. */
export async function loadVrchatFriends(force = false) {
  if (!force && (friendsLoaded || app.vrchatFriends.length > 0)) return;
  try {
    app.vrchatFriends = await vrchatFriends();
    friendsLoaded = true;
    for (const f of app.vrchatFriends) app.friendNames[f.id] = f.display_name;
    persistNames();
  } catch (e) {
    console.error("vrchatFriends failed", e);
  }
}

export async function loadVrchatLogin() {
  try {
    app.vrchatLogin = await vrchatStatus();
  } catch (e) {
    console.error("vrchatStatus failed", e);
  }
}

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

export async function setSleepPhase(phase: SleepPhase) {
  try {
    await setPhase(phase);
    if (app.state) app.state.sleep_phase = phase;
  } catch (e) {
    console.error("setPhase failed", e);
  }
}

async function tick() {
  try {
    app.state = await getState();
    app.connected = true;
    if (!app.config) app.config = await getConfig();
    // Cheap in-process command; keeps login state fresh (e.g. after the
    // startup session restore validates).
    app.vrchatLogin = await vrchatStatus();
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
