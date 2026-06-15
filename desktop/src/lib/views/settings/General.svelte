<script lang="ts">
  import { app, save } from "$lib/state.svelte";
  import { launchOverlay } from "$lib/api";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";

  let launching = $state(false);
  let result = $state<"" | "ok" | "fail">("");

  async function launch() {
    launching = true;
    result = "";
    try {
      result = (await launchOverlay()) ? "ok" : "fail";
      setTimeout(() => (result = ""), 4000);
    } catch {
      result = "fail";
    } finally {
      launching = false;
    }
  }
</script>

{#if app.config}
  <GlassCard title="In-Headset Overlay">
    <div class="rows">
      <div class="row">
        <div class="txt">
          <span class="t">Auto-launch with VR</span>
          <span class="d">Start the in-headset overlay automatically when a Monado VR session begins (it closes itself when VR ends)</span>
        </div>
        <Toggle bind:checked={app.config.auto_launch_overlay} label="Auto-launch overlay" onchange={save} />
      </div>
      <div class="row">
        <div class="txt">
          <span class="t">Launch now</span>
          <span class="d">Start the overlay manually (it'll exit if no VR runtime is running)</span>
        </div>
        <button class="btn tonal state-layer" disabled={launching} onclick={launch}>
          {launching ? "Launching…" : result === "ok" ? "Launched ✓" : result === "fail" ? "Overlay not found" : "Launch VR overlay"}
        </button>
      </div>
    </div>
  </GlassCard>

  <GlassCard title="General">
    <div class="rows">
      <div class="row">
        <div class="txt">
          <span class="t">Block game input over panels</span>
          <span class="d">Stop the game receiving controller input while you point at the in-headset panels</span>
        </div>
        <Toggle bind:checked={app.config.block_game_input} label="Block game input" onchange={save} />
      </div>
    </div>
  </GlassCard>
{/if}

<style>
  .rows {
    display: flex;
    flex-direction: column;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 0;
    border-top: 1px solid hsl(var(--glass-border) / 0.06);
  }
  .row:first-child {
    border-top: none;
  }
  .txt {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .txt .t {
    font-weight: 600;
    font-size: 14.5px;
  }
  .txt .d {
    color: hsl(var(--muted-foreground));
    font-size: 12.5px;
    max-width: 520px;
  }
  .btn {
    height: 36px;
    padding: 0 16px;
    font-size: 13px;
    flex: none;
  }
</style>
