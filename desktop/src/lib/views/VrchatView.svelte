<script lang="ts">
  import { app, save } from "$lib/state.svelte";
  import { testSound } from "$lib/api";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";

  const players = $derived(app.state?.player_count ?? 0);
  const world = $derived(app.state?.vrchat_world ?? null);
</script>

{#if app.config}
  <div class="view">
    <h2>VRChat</h2>
    <p class="lede">
      Reads the VRChat log (in VR or desktop mode) — no login needed. Keep this app running (it lives in the tray).
    </p>

    <div class="live glass">
      <div class="stat"><span class="n">{world ? players : "—"}</span><span class="l">players in world</span></div>
      <div class="world">{world ?? "Not in a world"}</div>
    </div>

    <GlassCard title="Join Notifications" desc="Play a sound when a player enters or leaves your world.">
      <div class="rows">
        <div class="row">
          <div class="txt"><span class="t">Enable</span></div>
          <Toggle bind:checked={app.config.vrchat.join_notifications.enabled} label="Join notifications" onchange={save} />
        </div>
        <div class="row" class:dim={!app.config.vrchat.join_notifications.enabled}>
          <div class="txt">
            <span class="t">Only when previously alone</span>
            <span class="d">Only notify if you were alone before the player joined / are alone after they left</span>
          </div>
          <Toggle bind:checked={app.config.vrchat.join_notifications.only_when_alone} label="Only when alone" onchange={save} />
        </div>
        <div class="row" class:dim={!app.config.vrchat.join_notifications.enabled}>
          <div class="txt">
            <span class="t">Only when sleep mode is enabled</span>
          </div>
          <Toggle bind:checked={app.config.vrchat.join_notifications.only_when_sleep} label="Only when sleeping" onchange={save} />
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

    <GlassCard title="Auto-Accept Invites & Status Automations">
      <p class="soon">
        Coming in a later update — these need a VRChat login. The toggles are already in the in-headset menu and persist
        here; the engine that acts on them is next.
      </p>
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
    max-width: 620px;
  }
  .live {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 22px 26px;
    border-radius: var(--radius-l);
  }
  .stat {
    display: flex;
    flex-direction: column;
  }
  .stat .n {
    font-size: 38px;
    font-weight: 700;
    line-height: 1;
  }
  .stat .l {
    color: hsl(var(--muted-foreground));
    font-size: 13px;
    margin-top: 4px;
  }
  .world {
    color: hsl(var(--foreground));
    font-weight: 600;
    text-align: right;
    max-width: 55%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
    line-height: 1.4;
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
  .soon {
    margin: 0;
    color: hsl(var(--muted-foreground));
    font-size: 13.5px;
    line-height: 1.5;
  }
</style>
