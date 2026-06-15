<script lang="ts">
  import { app, save, saveSoon } from "$lib/state.svelte";
  import { vrchatFriends } from "$lib/api";
  import type { Friend } from "$lib/types";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
  import Slider from "$lib/components/Slider.svelte";

  let friends = $state<Friend[]>([]);
  let loadingFriends = $state(false);
  let query = $state("");

  const filtered = $derived(
    friends.filter((f) => f.display_name.toLowerCase().includes(query.toLowerCase())),
  );

  async function loadFriends() {
    if (loadingFriends) return;
    loadingFriends = true;
    try {
      friends = await vrchatFriends();
    } catch (e) {
      console.error(e);
    } finally {
      loadingFriends = false;
    }
  }

  function toggleFriend(id: string, checked: boolean) {
    const ids = app.config!.vrchat.auto_accept.player_ids;
    const i = ids.indexOf(id);
    if (checked && i < 0) ids.push(id);
    else if (!checked && i >= 0) ids.splice(i, 1);
    save();
  }
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

      <div class="row" class:dim={!aa.enabled}>
        <div class="txt">
          <span class="t">List mode</span>
          <span class="d">{aa.list_mode === "whitelist" ? "Accept only from listed friends" : "Accept from everyone except listed friends"}</span>
        </div>
        <select bind:value={aa.list_mode} onchange={save}>
          <option value="whitelist">Whitelist</option>
          <option value="blacklist">Blacklist</option>
        </select>
      </div>

      <div class="row" class:dim={!aa.enabled}>
        <div class="txt"><span class="t">Only when sleep mode is enabled</span></div>
        <Toggle bind:checked={aa.only_when_sleep} label="Only when sleeping" onchange={save} />
      </div>

      <div class="row" class:dim={!aa.enabled}>
        <div class="txt">
          <span class="t">Only when fewer than N players</span>
          <span class="d">Counts everyone in your world, including you</span>
        </div>
        <Toggle bind:checked={aa.max_players_enabled} label="Limit by players" onchange={save} />
      </div>
      {#if aa.max_players_enabled}
        <div class="row slider" class:dim={!aa.enabled}>
          <Slider label="Max players" min={1} max={40} editable bind:value={aa.max_players} onchange={saveSoon} />
        </div>
      {/if}
    </div>

    <div class="list-head" class:dim={!aa.enabled}>
      <strong>{aa.list_mode === "whitelist" ? "Whitelist" : "Blacklist"}</strong>
      <span class="count">{aa.player_ids.length} selected</span>
      <button class="btn tonal state-layer" onclick={loadFriends} disabled={loadingFriends}>
        {loadingFriends ? "Loading…" : friends.length ? "Refresh friends" : "Load friends"}
      </button>
    </div>
    {#if friends.length}
      <input class="search" type="text" placeholder="Search friends…" bind:value={query} />
      <div class="friends" class:dim={!aa.enabled}>
        {#each filtered as f (f.id)}
          <label class="friend">
            <input
              type="checkbox"
              checked={aa.player_ids.includes(f.id)}
              onchange={(e) => toggleFriend(f.id, e.currentTarget.checked)}
            />
            {f.display_name}
          </label>
        {/each}
      </div>
    {/if}
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
    max-width: 480px;
  }
  select,
  .search {
    height: 36px;
    border-radius: var(--radius-s);
    border: 1px solid hsl(var(--glass-border) / 0.1);
    background: hsl(var(--glass-bg) / 0.5);
    color: hsl(var(--foreground));
    padding: 0 12px;
    font-size: 14px;
    font-family: inherit;
  }
  select:focus,
  .search:focus {
    outline: none;
    border-color: hsl(var(--primary) / 0.6);
  }
  .list-head {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 18px;
  }
  .list-head .count {
    color: hsl(var(--muted-foreground));
    font-size: 13px;
    flex: 1;
  }
  .list-head .btn {
    height: 32px;
    padding: 0 14px;
    font-size: 13px;
  }
  .list-head.dim {
    opacity: 0.4;
  }
  .search {
    width: 100%;
    margin-top: 12px;
  }
  .friends {
    margin-top: 10px;
    max-height: 260px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .friends.dim {
    opacity: 0.4;
    pointer-events: none;
  }
  .friend {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 8px;
    border-radius: var(--radius-s);
    font-size: 14px;
    cursor: pointer;
  }
  .friend:hover {
    background: hsl(var(--glass-border) / 0.06);
  }
  .friend input {
    width: 17px;
    height: 17px;
    accent-color: hsl(var(--primary));
  }
</style>
