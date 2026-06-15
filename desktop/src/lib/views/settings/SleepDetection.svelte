<script lang="ts">
  import { app, save, saveSoon } from "$lib/state.svelte";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
  import Slider from "$lib/components/Slider.svelte";

  const sensitivities = [
    { v: "low", l: "Low — needs near-perfect stillness" },
    { v: "medium", l: "Medium — recommended" },
    { v: "high", l: "High — sleeps with small movements" },
  ];
</script>

{#if app.config}
  {@const sleep = app.config.sleep}
  <GlassCard
    title="Sleep Detection"
    desc="Enter sleep mode automatically once your head stays still for a while. A cancelable countdown appears in-headset first — move your head or press any controller button to stay awake. It only ever puts you to sleep, never wakes you."
  >
    <div class="rows">
      <div class="row">
        <div class="txt"><span class="t">Enable detection</span></div>
        <Toggle bind:checked={sleep.detection_enabled} label="Sleep detection" onchange={save} />
      </div>

      <div class="row" class:dim={!sleep.detection_enabled}>
        <div class="txt">
          <span class="t">Sensitivity</span>
          <span class="d">How forgiving stillness detection is</span>
        </div>
        <select bind:value={sleep.detection_sensitivity} onchange={save}>
          {#each sensitivities as s (s.v)}<option value={s.v}>{s.l}</option>{/each}
        </select>
      </div>

      <div class="row slider" class:dim={!sleep.detection_enabled}>
        <Slider
          label="Minutes of stillness before sleeping"
          min={1}
          max={60}
          editable
          bind:value={sleep.detection_minutes}
          onchange={saveSoon}
        />
      </div>

      <div class="row" class:dim={!sleep.detection_enabled}>
        <div class="txt">
          <span class="t">Watch all the time</span>
          <span class="d">Off = only watch inside a time window</span>
        </div>
        <Toggle bind:checked={sleep.detection_always} label="Always watch" onchange={save} />
      </div>

      <div class="row" class:dim={!sleep.detection_enabled || sleep.detection_always}>
        <div class="txt"><span class="t">Watch from</span></div>
        <input type="time" bind:value={sleep.detect_start} onchange={save} />
      </div>
      <div class="row" class:dim={!sleep.detection_enabled || sleep.detection_always}>
        <div class="txt"><span class="t">Until</span></div>
        <input type="time" bind:value={sleep.detect_end} onchange={save} />
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
  .row.slider {
    display: block;
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
  select {
    height: 36px;
    border-radius: var(--radius-s);
    border: 1px solid hsl(var(--glass-border) / 0.1);
    background: hsl(var(--glass-bg) / 0.5);
    color: hsl(var(--foreground));
    padding: 0 12px;
    font-size: 14px;
    font-family: inherit;
  }
  select:focus {
    outline: none;
    border-color: hsl(var(--primary) / 0.6);
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
