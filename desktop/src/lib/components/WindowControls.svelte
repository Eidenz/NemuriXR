<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  const win = getCurrentWindow();
</script>

<div class="wc">
  <button class="wcb state-layer" aria-label="Minimize" onclick={() => win.minimize()}>
    <svg viewBox="0 0 24 24" width="16" height="16"><path fill="currentColor" d="M5 11h14v2H5z" /></svg>
  </button>
  <button class="wcb close" aria-label="Close" onclick={() => win.close()}>
    <svg viewBox="0 0 24 24" width="16" height="16"
      ><path
        fill="currentColor"
        d="M19 6.4 17.6 5 12 10.6 6.4 5 5 6.4 10.6 12 5 17.6 6.4 19 12 13.4 17.6 19 19 17.6 13.4 12 19 6.4Z"
      /></svg
    >
  </button>
</div>

<style>
  .wc {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .wcb {
    width: 38px;
    height: 34px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    color: hsl(var(--muted-foreground));
    transition: background 0.12s var(--ease), color 0.12s var(--ease);
  }
  .wcb:hover {
    background: hsl(var(--glass-border) / 0.1);
  }
  .wcb.close:hover {
    background: hsl(var(--error) / 0.22);
    color: hsl(var(--error));
  }
</style>
