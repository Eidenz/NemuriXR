<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { app } from "$lib/state.svelte";

  const KEY = "nemurixr.updateDismissed";
  const ls = typeof localStorage !== "undefined" ? localStorage : null;

  let dismissed = $state(ls?.getItem(KEY) ?? "");
  // Show until dismissed; a newer version than the dismissed one shows again.
  const show = $derived(!!app.update && app.update.version !== dismissed);

  function view() {
    if (app.update?.url) openUrl(app.update.url);
  }
  function dismiss() {
    if (app.update) {
      dismissed = app.update.version;
      ls?.setItem(KEY, app.update.version);
    }
  }
</script>

{#if show && app.update}
  <div class="banner">
    <span class="msg">🎉 NemuriXR <strong>v{app.update.version}</strong> is available.</span>
    <div class="acts">
      <button class="view" onclick={view}>View release</button>
      <button class="x" aria-label="Dismiss" onclick={dismiss}>✕</button>
    </div>
  </div>
{/if}

<style>
  .banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 9px 18px;
    background: hsl(var(--primary) / 0.16);
    border-bottom: 1px solid hsl(var(--primary) / 0.28);
    font-size: 13.5px;
    flex: none;
  }
  .acts {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .view {
    background: hsl(var(--primary));
    color: #fff;
    border: none;
    border-radius: var(--radius-s);
    padding: 5px 12px;
    font-size: 13px;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
  }
  .x {
    background: none;
    border: none;
    color: hsl(var(--muted-foreground));
    cursor: pointer;
    font-size: 13px;
    line-height: 1;
    padding: 4px;
  }
  .x:hover {
    color: hsl(var(--foreground));
  }
</style>
