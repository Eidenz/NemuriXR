<script lang="ts">
  import { app, save, saveSoon } from "$lib/state.svelte";
  import { applyBrightness } from "$lib/api";
  import type { BrightnessLevel, SleepPhase } from "$lib/types";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
  import Slider from "$lib/components/Slider.svelte";

  // Fan control only exists on the Bigscreen Beyond (HID); the libmonado
  // fallback can't set fans.
  const beyond = $derived(app.state?.brightness_backend === "Bigscreen Beyond");

  // Phase → its brightness level (Awake uses on_wake, etc.).
  const cards: { key: SleepPhase; title: string; level: () => BrightnessLevel }[] = [
    { key: "awake", title: "When I wake up", level: () => app.config!.brightness.on_wake },
    { key: "prepare", title: "When I prepare to sleep", level: () => app.config!.brightness.on_prepare },
    { key: "sleep", title: "When I go to sleep", level: () => app.config!.brightness.on_sleep },
  ];
</script>

{#if app.config}
  <div class="view">
    <div class="head">
      <div>
        <h2>Brightness &amp; Fans</h2>
        <p>Each phase has its own brightness and fan level, with a fade time to ease into it.</p>
      </div>
      <Toggle bind:checked={app.config.brightness.enabled} label="Brightness automations" onchange={save} />
    </div>

    <div class="grid" class:off={!app.config.brightness.enabled}>
      {#each cards as c (c.key)}
        {@const lvl = c.level()}
        <GlassCard title={c.title}>
          <div class="sliders">
            <Slider label="Brightness" suffix="%" bind:value={lvl.brightness} onchange={saveSoon} />
            <Slider label="Fan speed" suffix="%" disabled={!beyond} bind:value={lvl.fan} onchange={saveSoon} />
            <Slider label="Fade time" suffix="s" max={120} editable bind:value={lvl.transition_seconds} onchange={saveSoon} />
          </div>
          <button class="btn tonal state-layer preview" onclick={() => applyBrightness(c.key)}>Preview on headset</button>
        </GlassCard>
      {/each}
    </div>

    {#if !beyond}
      <p class="note">
        Fan control needs a Bigscreen Beyond. On other headsets, brightness is set through libmonado and the fan
        sliders are ignored.
      </p>
    {/if}
  </div>
{/if}

<style>
  .view {
    display: flex;
    flex-direction: column;
    gap: 20px;
    max-width: 1000px;
    margin: 0 auto;
  }
  .head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }
  .head h2 {
    font-size: 22px;
  }
  .head p {
    margin: 6px 0 0;
    color: hsl(var(--muted-foreground));
    font-size: 13.5px;
    max-width: 560px;
    line-height: 1.45;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 18px;
    transition: opacity 0.2s var(--ease);
  }
  .grid.off {
    opacity: 0.45;
    pointer-events: none;
  }
  .sliders {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  .preview {
    margin-top: 20px;
    height: 34px;
    padding: 0 16px;
    font-size: 13px;
    width: 100%;
  }
  .note {
    color: hsl(var(--muted-foreground));
    font-size: 13px;
    background: hsl(var(--primary) / 0.1);
    border: 1px solid hsl(var(--primary) / 0.2);
    border-radius: var(--radius-m);
    padding: 12px 16px;
    line-height: 1.45;
  }
</style>
