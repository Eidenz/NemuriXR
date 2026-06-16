<script lang="ts">
  import { onMount } from "svelte";
  import { app, load, save, saveSoon } from "$lib/state.svelte";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
  import Slider from "$lib/components/Slider.svelte";

  const sensitivities = [
    { v: "low", l: "Low — needs near-perfect stillness" },
    { v: "medium", l: "Medium — recommended" },
    { v: "high", l: "High — sleeps with small movements" },
  ];

  // Calibration is captured in-headset, so re-read config on open to show the
  // latest saved poses.
  onMount(load);

  function deletePose(i: number) {
    if (!app.config) return;
    app.config.sleep.detection_poses.splice(i, 1);
    save();
  }
  function clearPoses() {
    if (!app.config) return;
    app.config.sleep.detection_poses = [];
    save();
  }
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

  <GlassCard
    title="Sleep Pose Calibration"
    desc="Optional. Calibrate the position you sleep in so detection only triggers in that posture — useful if you nap reclined or on your side. Without it, staying still in any position is enough."
  >
    <div class="rows">
      <div class="row">
        <div class="txt">
          <span class="t">Calibrated poses</span>
          <span class="d">
            {#if sleep.detection_poses.length === 0}
              None — stillness alone will trigger sleep
            {:else}
              {sleep.detection_poses.length} saved — detection only arms near one of these
            {/if}
          </span>
        </div>
        {#if sleep.detection_poses.length > 0}
          <button class="clear" onclick={clearPoses}>Clear all</button>
        {/if}
      </div>

      {#if sleep.detection_poses.length > 0}
        <div class="poses">
          {#each sleep.detection_poses as pose, i (i)}
            <div class="pose">
              <input class="pose-name" type="text" bind:value={pose.name} placeholder="Pose name" onchange={save} />
              <button class="del" aria-label="Delete pose" title="Delete pose" onclick={() => deletePose(i)}>✕</button>
            </div>
          {/each}
        </div>
      {/if}

      <div class="row slider" class:dim={sleep.detection_poses.length === 0}>
        <Slider
          label="Pose match tolerance (degrees)"
          min={10}
          max={50}
          editable
          bind:value={sleep.detection_pose_tolerance}
          onchange={saveSoon}
        />
      </div>

      <p class="hint">
        To add a pose, put on the headset and open the menu (double-tap <strong>A</strong> on the right controller), then choose
        <strong>Calibrate sleep pose</strong>. Lie down the way you sleep and capture — add one per position you use, then name them here.
      </p>
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
  .poses {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 0 4px;
  }
  .pose {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .pose-name {
    flex: 1;
    min-width: 0;
    height: 36px;
    border-radius: var(--radius-s);
    border: 1px solid hsl(var(--glass-border) / 0.1);
    background: hsl(var(--glass-bg) / 0.5);
    color: hsl(var(--foreground));
    padding: 0 12px;
    font-size: 14px;
    font-family: inherit;
  }
  .pose-name:focus {
    outline: none;
    border-color: hsl(var(--primary) / 0.6);
  }
  .del {
    flex: none;
    width: 36px;
    height: 36px;
    border-radius: var(--radius-s);
    border: 1px solid hsl(var(--glass-border) / 0.1);
    background: hsl(var(--glass-bg) / 0.5);
    color: hsl(var(--muted-foreground));
    cursor: pointer;
    font-size: 13px;
    transition: color 0.15s var(--ease), border-color 0.15s var(--ease);
  }
  .del:hover {
    color: hsl(0 70% 68%);
    border-color: hsl(0 70% 68% / 0.5);
  }
  .clear {
    height: 36px;
    padding: 0 14px;
    border-radius: var(--radius-s);
    border: 1px solid hsl(var(--glass-border) / 0.12);
    background: hsl(var(--glass-bg) / 0.5);
    color: hsl(var(--foreground));
    font-size: 13.5px;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    transition: border-color 0.15s var(--ease), opacity 0.15s var(--ease);
  }
  .clear:hover:not(:disabled) {
    border-color: hsl(var(--primary) / 0.6);
  }
  .clear:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .hint {
    margin: 14px 0 0;
    color: hsl(var(--muted-foreground));
    font-size: 12.5px;
    line-height: 1.55;
  }
  .hint strong {
    color: hsl(var(--foreground));
  }
</style>
