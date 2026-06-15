<script lang="ts">
  import { app } from "$lib/state.svelte";
  const players = $derived(app.state?.player_count ?? 0);
  const world = $derived(app.state?.vrchat_world ?? null);
</script>

<div class="live glass">
  <div class="left">
    <span class="n">{world ? players : "—"}</span>
    <span class="l">players in world</span>
  </div>
  <span class="w">{world ?? "Not in a world"}</span>
</div>

<style>
  .live {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 20px;
    border-radius: var(--radius-m);
  }
  .left {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }
  .n {
    font-size: 26px;
    font-weight: 700;
    line-height: 1;
  }
  .l {
    color: hsl(var(--muted-foreground));
    font-size: 13px;
  }
  .w {
    color: hsl(var(--foreground));
    font-weight: 600;
    max-width: 55%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
