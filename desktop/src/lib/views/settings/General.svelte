<script lang="ts">
  import { app, save } from "$lib/state.svelte";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
</script>

{#if app.config}
  <GlassCard title="General">
    <div class="rows">
      <div class="row">
        <div class="txt">
          <span class="t">Block game input over panels</span>
          <span class="d">Stop the game receiving controller input while you point at the in-headset panels</span>
        </div>
        <Toggle bind:checked={app.config.block_game_input} label="Block game input" onchange={save} />
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
</style>
