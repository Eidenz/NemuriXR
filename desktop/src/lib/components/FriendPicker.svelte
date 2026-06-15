<script lang="ts">
  import { app, cacheFriendName } from "$lib/state.svelte";
  import type { Friend } from "$lib/types";

  interface Props {
    ids: string[];
    friends: Friend[];
    /** Load the friends list (cached); `force` re-fetches. Called only on demand. */
    onLoad?: (force: boolean) => Promise<void> | void;
    onchange?: () => void;
  }
  let { ids = $bindable(), friends, onLoad, onchange }: Props = $props();

  let searching = $state(false);
  let loading = $state(false);
  let query = $state("");
  let open = $state(false);

  // Friends list first, then the persisted name cache, then the raw id.
  const nameOf = (id: string) => friends.find((f) => f.id === id)?.display_name ?? app.friendNames[id] ?? id;

  const matches = $derived(
    query.trim()
      ? friends
          .filter((f) => !ids.includes(f.id) && f.display_name.toLowerCase().includes(query.toLowerCase()))
          .slice(0, 8)
      : [],
  );

  async function startSearch() {
    searching = true;
    if (friends.length === 0) {
      loading = true;
      await onLoad?.(false);
      loading = false;
    }
  }
  async function refresh() {
    loading = true;
    await onLoad?.(true);
    loading = false;
  }
  function add(f: Friend) {
    if (!ids.includes(f.id)) {
      ids.push(f.id);
      cacheFriendName(f.id, f.display_name);
      onchange?.();
    }
    query = "";
  }
  function remove(id: string) {
    const i = ids.indexOf(id);
    if (i >= 0) {
      ids.splice(i, 1);
      onchange?.();
    }
  }
</script>

<div class="picker">
  <div class="chips">
    {#each ids as id (id)}
      <span class="chip">
        {nameOf(id)}
        <button class="x" aria-label="Remove" onclick={() => remove(id)}>✕</button>
      </span>
    {/each}
    {#if ids.length === 0}
      <span class="empty">No one added yet.</span>
    {/if}
  </div>

  {#if !searching}
    <button class="add btn tonal state-layer" onclick={startSearch}>+ Add friend</button>
  {:else}
    <div class="searchbar">
      <div class="ac">
        <input
          type="text"
          placeholder={loading ? "Loading friends…" : "Search friends…"}
          bind:value={query}
          onfocus={() => (open = true)}
          onblur={() => setTimeout(() => (open = false), 150)}
        />
        {#if open && matches.length}
          <div class="menu glass">
            {#each matches as f (f.id)}
              <button class="opt" onmousedown={() => add(f)}>{f.display_name}</button>
            {/each}
          </div>
        {:else if open && query.trim() && !loading}
          <div class="menu glass"><div class="none">No matching friends — use ↻ to re-fetch</div></div>
        {/if}
      </div>
      {#if loading}
        <span class="spinner" aria-label="Loading friends"></span>
      {/if}
      <button class="link" onclick={refresh} disabled={loading} title="Re-fetch friends">↻</button>
      <button class="link" onclick={() => ((searching = false), (query = ""))}>Done</button>
    </div>
  {/if}
</div>

<style>
  .picker {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    min-height: 24px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 30px;
    padding: 0 6px 0 12px;
    border-radius: var(--radius-pill);
    background: hsl(var(--primary) / 0.16);
    border: 1px solid hsl(var(--primary) / 0.28);
    color: hsl(var(--foreground));
    font-size: 13px;
    font-weight: 600;
  }
  .chip .x {
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    color: hsl(var(--muted-foreground));
    font-size: 11px;
  }
  .chip .x:hover {
    background: hsl(var(--error) / 0.25);
    color: hsl(var(--error));
  }
  .empty {
    color: hsl(var(--muted-foreground));
    font-size: 13px;
  }
  .add {
    align-self: flex-start;
    height: 34px;
    padding: 0 14px;
    font-size: 13px;
  }
  .searchbar {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .ac {
    position: relative;
    flex: 1;
    max-width: 360px;
  }
  .ac input {
    width: 100%;
    height: 38px;
    border-radius: var(--radius-s);
    border: 1px solid hsl(var(--glass-border) / 0.1);
    background: hsl(var(--glass-bg) / 0.5);
    color: hsl(var(--foreground));
    padding: 0 12px;
    font-size: 14px;
    font-family: inherit;
  }
  .ac input:focus {
    outline: none;
    border-color: hsl(var(--primary) / 0.6);
  }
  .menu {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    z-index: 10;
    border-radius: var(--radius-s);
    padding: 4px;
    max-height: 240px;
    overflow-y: auto;
    box-shadow: 0 12px 30px hsl(245 40% 2% / 0.5);
  }
  .opt {
    display: block;
    width: 100%;
    text-align: left;
    padding: 8px 10px;
    border-radius: var(--radius-s);
    font-size: 14px;
    color: hsl(var(--foreground));
  }
  .opt:hover {
    background: hsl(var(--primary) / 0.18);
  }
  .none {
    padding: 8px 10px;
    color: hsl(var(--muted-foreground));
    font-size: 13px;
  }
  .link {
    color: hsl(var(--primary));
    font-size: 13px;
    font-weight: 600;
    padding: 6px 8px;
    border-radius: var(--radius-s);
  }
  .link:hover {
    background: hsl(var(--primary) / 0.12);
  }
  .link:disabled {
    opacity: 0.5;
  }
  .spinner {
    width: 18px;
    height: 18px;
    flex: none;
    border: 2px solid hsl(var(--glass-border) / 0.2);
    border-top-color: hsl(var(--primary));
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
