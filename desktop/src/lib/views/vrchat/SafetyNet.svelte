<script lang="ts">
  import { app, save } from "$lib/state.svelte";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
</script>

{#if app.config}
  {@const sn = app.config.safety_net}
  <GlassCard
    title="Auto-Sleep Safety Net"
    desc="For when you doze off without setting up. These run ONLY when sleep starts from motion detection — a manual sleep toggle never touches any of it."
  >
    <div class="rows">
      <div class="row">
        <div class="txt"><span class="t">Enable safety net</span></div>
        <Toggle bind:checked={sn.enabled} label="Safety net" onchange={save} />
      </div>

      <div class="row" class:dim={!sn.enabled}>
        <div class="txt">
          <span class="t">Apply a fallback pose</span>
          <span class="d">Uses your pose set from <strong>Sleeping Pose</strong> so you're not a pretzel</span>
        </div>
        <Toggle bind:checked={sn.pose} label="Fallback pose" onchange={save} />
      </div>
      <div class="row sub" class:dim={!sn.enabled || !sn.pose}>
        <div class="txt">
          <span class="t">Skip if trackers are connected</span>
          <span class="d">If FBT trackers are on, leave the pose to you</span>
        </div>
        <Toggle bind:checked={sn.pose_skip_if_trackers} label="Skip if trackers" onchange={save} />
      </div>
      <div class="row sub" class:dim={!sn.enabled || !sn.pose}>
        <div class="txt">
          <span class="t">Override an existing pose</span>
          <span class="d">Re-pose even if you're already in a GoGo pose (default off)</span>
        </div>
        <Toggle bind:checked={sn.pose_override_existing} label="Override existing pose" onchange={save} />
      </div>

      <div class="row" class:dim={!sn.enabled}>
        <div class="txt">
          <span class="t">Mute mic — device</span>
          <span class="d">Mute the microphone device (PipeWire); most reliable</span>
        </div>
        <Toggle bind:checked={sn.mute_device} label="Mute device mic" onchange={save} />
      </div>
      <div class="row" class:dim={!sn.enabled}>
        <div class="txt">
          <span class="t">Mute mic — in-game</span>
          <span class="d">Toggle VRChat's own mute over OSC (needs toggle-voice mode)</span>
        </div>
        <Toggle bind:checked={sn.mute_ingame} label="Mute in-game mic" onchange={save} />
      </div>
    </div>

    <p class="note">
      Everything here reverts when you wake. The pose uses the set configured under <strong>Sleeping Pose</strong>, and
      "already in a GoGo pose" is detected by reading VRChat over OSCQuery.
    </p>
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
  .row.sub {
    padding-left: 18px;
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
    max-width: 460px;
  }
  .txt .d strong {
    color: hsl(var(--foreground));
  }
  .note {
    margin: 16px 0 0;
    color: hsl(var(--muted-foreground));
    font-size: 12.5px;
    line-height: 1.5;
  }
  .note strong {
    color: hsl(var(--foreground));
  }
</style>
