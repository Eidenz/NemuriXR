<script lang="ts">
  import { app, save } from "$lib/state.svelte";
  import { testCommand } from "$lib/api";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
</script>

{#if app.config}
  {@const c = app.config.commands}
  <GlassCard
    title="Run Commands"
    desc="Run a shell command or app on each phase change — turn off smart lights, send a notification, suspend the PC, anything."
  >
    <div class="rows">
      <div class="row">
        <div class="txt"><span class="t">Enable commands</span></div>
        <Toggle bind:checked={c.enabled} label="Run commands" onchange={save} />
      </div>

      <div class="cmd" class:dim={!c.enabled}>
        <span class="lbl">When I wake up</span>
        <div class="line">
          <input type="text" placeholder="e.g. notify-send 'Good morning'" bind:value={c.on_wake} onchange={save} />
          <button class="btn tonal state-layer" onclick={() => testCommand("awake")}>Run now</button>
        </div>
      </div>

      <div class="cmd" class:dim={!c.enabled}>
        <span class="lbl">When I prepare to sleep</span>
        <div class="line">
          <input type="text" placeholder="e.g. my-lights dim" bind:value={c.on_prepare} onchange={save} />
          <button class="btn tonal state-layer" onclick={() => testCommand("prepare")}>Run now</button>
        </div>
      </div>

      <div class="cmd" class:dim={!c.enabled}>
        <span class="lbl">When I go to sleep</span>
        <div class="line">
          <input type="text" placeholder="e.g. my-lights off" bind:value={c.on_sleep} onchange={save} />
          <button class="btn tonal state-layer" onclick={() => testCommand("sleep")}>Run now</button>
        </div>
      </div>
    </div>

    <p class="note">
      Commands run through <code>sh -c</code> on this computer. Only enter commands you trust.
    </p>
  </GlassCard>
{/if}

<style>
  .rows {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 0;
    border-bottom: 1px solid hsl(var(--glass-border) / 0.06);
    margin-bottom: 6px;
  }
  .txt .t {
    font-weight: 600;
    font-size: 14.5px;
  }
  .cmd {
    display: flex;
    flex-direction: column;
    gap: 7px;
    padding: 10px 0;
  }
  .cmd.dim {
    opacity: 0.4;
    pointer-events: none;
  }
  .lbl {
    font-weight: 600;
    font-size: 13.5px;
  }
  .line {
    display: flex;
    gap: 8px;
  }
  input {
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
  input:focus {
    outline: none;
    border-color: hsl(var(--primary) / 0.6);
  }
  .line .btn {
    height: 36px;
    padding: 0 14px;
    font-size: 13px;
    flex: none;
  }
  .note {
    margin: 16px 0 0;
    color: hsl(var(--muted-foreground));
    font-size: 12.5px;
    line-height: 1.5;
  }
  .note code {
    font-family: var(--font-mono, monospace);
    font-size: 12px;
  }
</style>
