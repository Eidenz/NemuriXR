<script lang="ts">
  import { app, save, saveSoon } from "$lib/state.svelte";
  import { applyBrightness } from "$lib/api";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
  import Slider from "$lib/components/Slider.svelte";

  // Fan control only exists on the Bigscreen Beyond (HID); the libmonado
  // fallback can't set fans.
  const beyond = $derived(app.state?.brightness_backend === "Bigscreen Beyond");
</script>

{#if app.config}
  <div class="view">
    <div class="head">
      <div>
        <h2>Brightness &amp; Fans</h2>
        <p>Dim the headset (and slow its fans) when you go to sleep, and restore it when you wake.</p>
      </div>
      <Toggle bind:checked={app.config.brightness.enabled} onchange={save} />
    </div>

    <div class="grid" class:off={!app.config.brightness.enabled}>
      <GlassCard title="When I go to sleep">
        <div class="sliders">
          <Slider label="Brightness" suffix="%" bind:value={app.config.brightness.on_sleep.brightness} onchange={saveSoon} />
          <Slider label="Fan speed" suffix="%" disabled={!beyond} bind:value={app.config.brightness.on_sleep.fan} onchange={saveSoon} />
        </div>
        <button class="btn tonal state-layer preview" onclick={() => applyBrightness("sleep")}>Preview on headset</button>
      </GlassCard>
      <GlassCard title="When I wake up">
        <div class="sliders">
          <Slider label="Brightness" suffix="%" bind:value={app.config.brightness.on_wake.brightness} onchange={saveSoon} />
          <Slider label="Fan speed" suffix="%" disabled={!beyond} bind:value={app.config.brightness.on_wake.fan} onchange={saveSoon} />
        </div>
        <button class="btn tonal state-layer preview" onclick={() => applyBrightness("wake")}>Preview on headset</button>
      </GlassCard>
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
    max-width: 880px;
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
    grid-template-columns: 1fr 1fr;
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
    gap: 22px;
  }
  .preview {
    margin-top: 20px;
    height: 34px;
    padding: 0 16px;
    font-size: 13px;
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
