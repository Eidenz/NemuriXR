<script lang="ts">
  import { app, save } from "$lib/state.svelte";
  import { testSound } from "$lib/api";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
</script>

{#if app.config}
  {@const n = app.config.vrchat.join_notifications}
  <GlassCard title="Join Notifications" desc="Play a sound when a player enters or leaves your world. Reads the VRChat log — no login required.">
    <div class="rows">
      <div class="row">
        <div class="txt"><span class="t">Enable</span></div>
        <Toggle bind:checked={n.enabled} label="Join notifications" onchange={save} />
      </div>
      <div class="row" class:dim={!n.enabled}>
        <div class="txt">
          <span class="t">Only when previously alone</span>
          <span class="d">Only notify if you were alone before the player joined / are alone after they left</span>
        </div>
        <Toggle bind:checked={n.only_when_alone} label="Only when alone" onchange={save} />
      </div>
      <div class="row" class:dim={!n.enabled}>
        <div class="txt"><span class="t">Only when sleep mode is enabled</span></div>
        <Toggle bind:checked={n.only_when_sleep} label="Only when sleeping" onchange={save} />
      </div>
      <div class="row">
        <div class="txt"><span class="t">Preview sounds</span></div>
        <div class="btns">
          <button class="btn tonal state-layer" onclick={() => testSound("join")}>Test join</button>
          <button class="btn tonal state-layer" onclick={() => testSound("leave")}>Test leave</button>
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
    max-width: 520px;
  }
  .btns {
    display: flex;
    gap: 8px;
  }
  .btns .btn {
    height: 34px;
    padding: 0 14px;
    font-size: 13px;
  }
</style>
