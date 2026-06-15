<script lang="ts">
  import { app, save } from "$lib/state.svelte";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
</script>

{#if app.config}
  {@const sleep = app.config.sleep}
  <div class="view">
    <h2>Settings</h2>

    <GlassCard
      title="Sleep Schedule"
      desc="Automatically enter and leave sleep mode at set times. It fires at the moment — a manual toggle still overrides until the next scheduled time."
    >
      <div class="rows">
        <div class="row">
          <div class="txt"><span class="t">Enable schedule</span></div>
          <Toggle bind:checked={sleep.schedule_enabled} label="Sleep schedule" onchange={save} />
        </div>
        <div class="row" class:dim={!sleep.schedule_enabled}>
          <div class="txt"><span class="t">Go to sleep at</span></div>
          <input type="time" bind:value={sleep.sleep_at} onchange={save} />
        </div>
        <div class="row" class:dim={!sleep.schedule_enabled}>
          <div class="txt"><span class="t">Wake up at</span></div>
          <input type="time" bind:value={sleep.wake_at} onchange={save} />
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
  </div>
{/if}

<style>
  .view {
    display: flex;
    flex-direction: column;
    gap: 18px;
    max-width: 880px;
    margin: 0 auto;
  }
  h2 {
    font-size: 22px;
  }
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
  .row.dim {
    opacity: 0.4;
    pointer-events: none;
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
  input[type="time"] {
    height: 36px;
    border-radius: var(--radius-s);
    border: 1px solid hsl(var(--glass-border) / 0.1);
    background: hsl(var(--glass-bg) / 0.5);
    color: hsl(var(--foreground));
    padding: 0 12px;
    font-size: 14px;
    font-family: inherit;
    color-scheme: dark;
  }
  input[type="time"]:focus {
    outline: none;
    border-color: hsl(var(--primary) / 0.6);
  }
</style>
