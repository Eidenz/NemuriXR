<script lang="ts">
  import { app, save, saveSoon, loadVrchatFriends } from "$lib/state.svelte";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
  import Slider from "$lib/components/Slider.svelte";
  import FriendPicker from "$lib/components/FriendPicker.svelte";
</script>

<GlassCard
  title="Auto-Accept Invites"
  desc="Automatically accept invite requests in real time, limited to chosen friends and your world's player count."
>
  {#if !app.vrchatLogin.logged_in}
    <p class="msg">Sign in under <strong>Settings → VRChat Account</strong> to use auto-accept.</p>
  {:else if app.config}
    {@const aa = app.config.vrchat.auto_accept}
    <div class="rows">
      <div class="row">
        <div class="txt"><span class="t">Enable</span></div>
        <Toggle bind:checked={aa.enabled} label="Auto-accept" onchange={save} />
      </div>

      <div class="row">
        <div class="txt">
          <span class="t">List mode</span>
          <span class="d">
            {aa.list_mode === "whitelist"
              ? "Accept only from listed friends"
              : "Accept from everyone except listed friends"}
          </span>
        </div>
        <select bind:value={aa.list_mode} onchange={save}>
          <option value="whitelist">Whitelist</option>
          <option value="blacklist">Blacklist</option>
        </select>
      </div>

      <div class="row">
        <div class="txt"><span class="t">Only when sleep mode is enabled</span></div>
        <Toggle bind:checked={aa.only_when_sleep} label="Only when sleeping" onchange={save} />
      </div>

      <div class="row">
        <div class="txt">
          <span class="t">Only when fewer than N players</span>
          <span class="d">Counts everyone in your world, including you</span>
        </div>
        <Toggle bind:checked={aa.max_players_enabled} label="Limit by players" onchange={save} />
      </div>
      {#if aa.max_players_enabled}
        <div class="row slider">
          <Slider label="Max players" min={1} max={40} editable bind:value={aa.max_players} onchange={saveSoon} />
        </div>
      {/if}
    </div>

    <div class="list-head">
      <strong>{aa.list_mode === "whitelist" ? "Whitelist" : "Blacklist"}</strong>
      <span class="count">{aa.player_ids.length} added</span>
    </div>
    <FriendPicker bind:ids={aa.player_ids} friends={app.vrchatFriends} onLoad={loadVrchatFriends} onchange={save} />
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
    padding-top: 4px;
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
    max-width: 480px;
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
  .list-head {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 20px 0 12px;
  }
  .list-head strong {
    font-size: 14.5px;
  }
  .list-head .count {
    color: hsl(var(--muted-foreground));
    font-size: 13px;
  }
</style>
