<script lang="ts">
  import { app, stopAlarmNow } from "$lib/state.svelte";

  // The wake-up alarm loops until dismissed; show a prominent Stop while it rings.
  const ringing = $derived(app.state?.alarm_active ?? false);
</script>

{#if ringing}
  <div class="banner" role="alert">
    <span class="msg">⏰ <strong>Wake up!</strong> The alarm is ringing.</span>
    <button class="stop" onclick={stopAlarmNow}>Stop alarm</button>
  </div>
{/if}

<style>
  .banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 11px 18px;
    background: hsl(0 70% 45% / 0.22);
    border-bottom: 1px solid hsl(0 75% 55% / 0.4);
    font-size: 14px;
    flex: none;
  }
  .msg {
    color: hsl(var(--foreground));
  }
  .stop {
    background: hsl(0 72% 52%);
    color: #fff;
    border: none;
    border-radius: var(--radius-s);
    padding: 8px 18px;
    font-size: 14px;
    font-weight: 700;
    font-family: inherit;
    cursor: pointer;
    flex: none;
    transition: background 0.15s var(--ease);
  }
  .stop:hover {
    background: hsl(0 72% 58%);
  }
</style>
