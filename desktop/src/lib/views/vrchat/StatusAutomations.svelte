<script lang="ts">
  import { app, save, saveSoon } from "$lib/state.svelte";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
  import Slider from "$lib/components/Slider.svelte";

  const statuses = [
    { v: "join_me", l: "Join Me" },
    { v: "active", l: "Active" },
    { v: "ask_me", l: "Ask Me" },
    { v: "busy", l: "Busy" },
  ];
</script>

<GlassCard
  title="Status Automations"
  desc="Automatically change your VRChat status based on how many players are in your world — e.g. switch to Ask Me once it fills up."
>
  {#if !app.vrchatLogin.logged_in}
    <p class="msg">Sign in under <strong>Settings → VRChat Account</strong> to use status automations.</p>
  {:else if app.config}
    {@const sa = app.config.vrchat.status_automations}
    <div class="rows">
      <div class="row">
        <div class="txt"><span class="t">Enable</span></div>
        <Toggle bind:checked={sa.enabled} label="Status automations" onchange={save} />
      </div>

      <div class="row slider" class:dim={!sa.enabled}>
        <Slider label="Player limit (includes you)" min={1} max={40} editable bind:value={sa.player_limit} onchange={saveSoon} />
      </div>

      <div class="row" class:dim={!sa.enabled}>
        <div class="txt"><span class="t">Below the limit</span><span class="d">Status when there's room</span></div>
        <select bind:value={sa.below_status} onchange={save}>
          {#each statuses as s (s.v)}<option value={s.v}>{s.l}</option>{/each}
        </select>
      </div>

      <div class="row" class:dim={!sa.enabled}>
        <div class="txt"><span class="t">At the limit or above</span><span class="d">Status when it's full</span></div>
        <select bind:value={sa.at_or_above_status} onchange={save}>
          {#each statuses as s (s.v)}<option value={s.v}>{s.l}</option>{/each}
        </select>
      </div>

      <div class="row" class:dim={!sa.enabled}>
        <div class="txt"><span class="t">Only when sleep mode is enabled</span></div>
        <Toggle bind:checked={sa.only_when_sleep} label="Only when sleeping" onchange={save} />
      </div>
    </div>
  {/if}
</GlassCard>

<style>
  .msg {
    margin: 0;
    color: hsl(var(--muted-foreground));
    font-size: 13.5px;
    line-height: 1.5;
  }
  .msg strong {
    color: hsl(var(--foreground));
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
</style>
