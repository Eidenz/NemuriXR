<script lang="ts">
  import { app } from "$lib/state.svelte";
  import GlassCard from "$lib/components/GlassCard.svelte";
</script>

<GlassCard
  title="Auto-Accept Invites"
  desc="Automatically accept invite requests from chosen friends, limited by how many players are already in your world."
>
  {#if !app.vrchatLogin.logged_in}
    <p class="msg">
      Sign in to your VRChat account under <strong>Settings → VRChat Account</strong> to use auto-accept.
    </p>
  {:else}
    <p class="msg">
      Signed in as <strong>{app.vrchatLogin.username}</strong>. The auto-accept engine (whitelist + player-count limit,
      accepting via the VRChat pipeline) is the next step.
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
