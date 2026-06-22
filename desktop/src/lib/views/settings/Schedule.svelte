<script lang="ts">
  import { app, save } from "$lib/state.svelte";
  import { testAlarm } from "$lib/api";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
</script>

{#if app.config}
  {@const sleep = app.config.sleep}
  <GlassCard
    title="Auto-Sleep"
    desc="Automatically enter sleep mode at a set time. It fires at the moment — a manual toggle still overrides until the next scheduled time. Independent of wake-up below."
  >
    <div class="rows">
      <div class="row">
        <div class="txt"><span class="t">Auto-sleep at a set time</span></div>
        <Toggle bind:checked={sleep.schedule_enabled} label="Auto-sleep" onchange={save} />
      </div>
      <div class="row" class:dim={!sleep.schedule_enabled}>
        <div class="txt"><span class="t">Go to sleep at</span></div>
        <input type="time" bind:value={sleep.sleep_at} onchange={save} />
      </div>
    </div>
  </GlassCard>

  <GlassCard
    title="Auto-Wake"
    desc="Automatically wake at a set time, with an optional alarm. Works without auto-sleep. Brightness eases in using the wake Fade time on the Brightness tab — same as a manual wake."
  >
    <div class="rows">
      <div class="row">
        <div class="txt"><span class="t">Auto-wake at a set time</span></div>
        <Toggle bind:checked={sleep.wake.enabled} label="Auto-wake" onchange={save} />
      </div>
      <div class="row" class:dim={!sleep.wake.enabled}>
        <div class="txt"><span class="t">Wake up at</span></div>
        <input type="time" bind:value={sleep.wake_at} onchange={save} />
      </div>
      <div class="row" class:dim={!sleep.wake.enabled}>
        <div class="txt">
          <span class="t">Play an alarm sound</span>
          <span class="d">Rings on a loop once you've woken, until you tap Stop (in the headset or here)</span>
        </div>
        <Toggle bind:checked={sleep.wake.alarm_enabled} label="Alarm" onchange={save} />
      </div>
      <div class="row" class:dim={!sleep.wake.enabled || !sleep.wake.alarm_enabled}>
        <div class="txt">
          <span class="t">Alarm sound</span>
          <span class="d">Path to a sound file — leave empty for the default chime</span>
        </div>
        <div class="alarm">
          <input type="text" placeholder="/path/to/alarm.ogg" bind:value={sleep.wake.alarm_sound} onchange={save} />
          <button class="btn tonal state-layer" onclick={testAlarm}>Test</button>
        </div>
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
  }
  .alarm {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .alarm .btn {
    height: 36px;
    padding: 0 14px;
    font-size: 13px;
    flex: none;
  }
  input {
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
  input[type="text"] {
    width: 260px;
  }
  input:focus {
    outline: none;
    border-color: hsl(var(--primary) / 0.6);
  }
</style>
