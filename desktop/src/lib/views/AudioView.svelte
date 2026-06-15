<script lang="ts">
  import { app, save, saveSoon } from "$lib/state.svelte";
  import { applyAudio } from "$lib/api";
  import type { AudioLevel, SleepPhase } from "$lib/types";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
  import Slider from "$lib/components/Slider.svelte";

  const cards: { key: SleepPhase; title: string; level: () => AudioLevel }[] = [
    { key: "awake", title: "When I wake up", level: () => app.config!.audio.on_wake },
    { key: "prepare", title: "When I prepare to sleep", level: () => app.config!.audio.on_prepare },
    { key: "sleep", title: "When I go to sleep", level: () => app.config!.audio.on_sleep },
  ];
</script>

{#if app.config}
  <div class="view">
    <div class="head">
      <div>
        <h2>Audio Volume</h2>
        <p>
          Set the output volume and mic for each phase — e.g. drop the volume and mute yourself when you sleep.
          It controls whichever device VRChat is using ({app.state?.audio_target ?? "auto-detected"}), falling back to
          your default device.
        </p>
      </div>
      <Toggle bind:checked={app.config.audio.enabled} label="Audio automations" onchange={save} />
    </div>

    <div class="grid">
      {#each cards as c (c.key)}
        {@const lvl = c.level()}
        <GlassCard title={c.title}>
          <div class="rows">
            <div class="row">
              <div class="txt"><span class="t">Set output volume</span></div>
              <Toggle bind:checked={lvl.set_volume} label="Set volume" onchange={save} />
            </div>
            {#if lvl.set_volume}
              <div class="row slider">
                <Slider label="Output volume" suffix="%" bind:value={lvl.volume} onchange={saveSoon} />
              </div>
            {/if}

            <div class="row">
              <div class="txt"><span class="t">Control microphone</span></div>
              <Toggle bind:checked={lvl.set_mic} label="Control mic" onchange={save} />
            </div>
            {#if lvl.set_mic}
              <div class="row">
                <div class="txt"><span class="t">Mute microphone</span></div>
                <Toggle bind:checked={lvl.mic_muted} label="Mute mic" onchange={save} />
              </div>
            {/if}
          </div>
          <button class="btn tonal state-layer preview" onclick={() => applyAudio(c.key)}>Preview now</button>
        </GlassCard>
      {/each}
    </div>

    <p class="note">
      Uses PipeWire/PulseAudio (<code>pactl</code>). NemuriXR detects VRChat's audio stream to find which device it
      plays through and adjusts that device; if VRChat isn't running it falls back to your system default device.
    </p>
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
    max-width: 620px;
    line-height: 1.45;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 18px;
  }
  .rows {
    display: flex;
    flex-direction: column;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 12px 0;
    border-top: 1px solid hsl(var(--glass-border) / 0.06);
  }
  .row:first-child {
    border-top: none;
  }
  .row.slider {
    display: block;
    padding-top: 6px;
  }
  .txt .t {
    font-weight: 600;
    font-size: 14px;
  }
  .preview {
    margin-top: 18px;
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
  .note code {
    font-family: var(--font-mono, monospace);
    font-size: 12px;
  }
</style>
