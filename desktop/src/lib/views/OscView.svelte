<script lang="ts">
  import { app, save, saveSoon } from "$lib/state.svelte";
  import { sendOsc } from "$lib/api";
  import type { OscMessage } from "$lib/types";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import OscRow from "$lib/components/OscRow.svelte";
  import Toggle from "$lib/components/Toggle.svelte";

  const target = $derived(app.state?.osc_target ?? null);

  function addMsg(list: OscMessage[]) {
    list.push({ address: "/avatar/parameters/", args: [{ kind: "bool", value: true }], delay_ms: 0 });
    save();
  }
  function removeMsg(list: OscMessage[], i: number) {
    list.splice(i, 1);
    save();
  }
</script>

{#if app.config}
  {@const osc = app.config.osc}
  <div class="view">
    <h2>OSC Automations</h2>
    <p class="lede">
      Send OSC messages to VRChat when you sleep and wake — e.g. flip an avatar parameter to trigger a sleep animation.
      Each message can wait a delay, so a list runs as a sequence. VRChat's OSC must be enabled (it is by default).
    </p>

    {#each [{ key: "sleep" as const, title: "When I go to sleep", list: osc.on_sleep }, { key: "wake" as const, title: "When I wake up", list: osc.on_wake }] as group (group.key)}
      <GlassCard title={group.title}>
        <div class="msgs">
          {#each group.list as msg, i (i)}
            <OscRow message={msg} onchange={saveSoon} onremove={() => removeMsg(group.list, i)} />
          {/each}
          {#if group.list.length === 0}
            <p class="empty">No messages — nothing is sent when you {group.key === "sleep" ? "go to sleep" : "wake up"}.</p>
          {/if}
        </div>
        <div class="actions">
          <button class="btn tonal state-layer" onclick={() => addMsg(group.list)}>+ Add message</button>
          <button class="btn tonal state-layer" disabled={group.list.length === 0} onclick={() => sendOsc(group.key)}>
            Send now
          </button>
        </div>
      </GlassCard>
    {/each}

    <GlassCard title="Connection">
      <div class="rows">
        <div class="row">
          <div class="txt">
            <span class="t">Auto-discover VRChat (OSCQuery)</span>
            <span class="d">Find VRChat's OSC port automatically over the network</span>
          </div>
          <Toggle bind:checked={osc.use_oscquery} label="Use OSCQuery" onchange={save} />
        </div>
        <div class="row">
          <div class="txt"><span class="t">Current target</span><span class="d">Where messages are sent right now</span></div>
          <span class="target">{target ?? "not resolved"}</span>
        </div>
        <div class="row" class:dim={osc.use_oscquery}>
          <div class="txt"><span class="t">Manual host / port</span><span class="d">Used when OSCQuery is off (or as fallback)</span></div>
          <div class="hp">
            <input type="text" bind:value={osc.host} oninput={saveSoon} />
            <input type="number" min="1" max="65535" bind:value={osc.port} oninput={saveSoon} />
          </div>
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
  .lede {
    margin: -8px 0 0;
    color: hsl(var(--muted-foreground));
    font-size: 13.5px;
    line-height: 1.45;
    max-width: 640px;
  }
  .msgs {
    display: flex;
    flex-direction: column;
  }
  .empty {
    color: hsl(var(--muted-foreground));
    font-size: 13px;
    margin: 4px 0;
  }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 14px;
  }
  .actions .btn {
    height: 34px;
    padding: 0 14px;
    font-size: 13px;
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
  }
  .target {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .hp {
    display: flex;
    gap: 8px;
  }
  .hp input {
    height: 34px;
    border-radius: var(--radius-s);
    border: 1px solid hsl(var(--glass-border) / 0.1);
    background: hsl(var(--glass-bg) / 0.5);
    color: hsl(var(--foreground));
    padding: 0 10px;
    font-size: 13px;
    font-family: inherit;
  }
  .hp input[type="text"] {
    width: 130px;
  }
  .hp input[type="number"] {
    width: 84px;
  }
</style>
