<script lang="ts">
  import { app } from "$lib/state.svelte";
  import GlassCard from "$lib/components/GlassCard.svelte";
</script>

<GlassCard
  title="Status Automations"
  desc="Automatically change your VRChat status (join me / ask me / busy) based on how many players are in your world."
>
  {#if !app.vrchatLogin.logged_in}
    <p class="msg">
      Sign in to your VRChat account under <strong>Settings → VRChat Account</strong> to use status automations.
    </p>
  {:else}
    <p class="msg">
      Signed in as <strong>{app.vrchatLogin.username}</strong>. The status-automation engine is coming alongside
      auto-accept.
    </p>
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
</style>
