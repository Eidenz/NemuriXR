<script lang="ts">
  import { app, setSleepPhase } from "$lib/state.svelte";
  import GlassCard from "$lib/components/GlassCard.svelte";

  const phase = $derived(app.state?.sleep_phase ?? "awake");
  const players = $derived(app.state?.player_count ?? 0);
  const world = $derived(app.state?.vrchat_world ?? null);
  const backend = $derived(app.state?.brightness_backend ?? null);
  const status = $derived(phase === "sleep" ? "Active" : phase === "prepare" ? "Preparing" : "Inactive");

  // The big card turns sleep on (from awake or prepare); only an active sleep
  // toggles back to awake.
  const toggleCard = () => setSleepPhase(phase === "sleep" ? "awake" : "sleep");
</script>

<div class="view">
  <button class="sleepcard glass" class:sleep={phase === "sleep"} class:prepare={phase === "prepare"} onclick={toggleCard}>
    <svg viewBox="0 0 24 24" width="56" height="56" aria-hidden="true">
      <path fill="currentColor" d="M12.7 2a8 8 0 1 0 9.3 9.3 7 7 0 0 1-9.3-9.3Z" />
    </svg>
    <div class="txt">
      <span class="k">Sleep Mode</span>
      <span class="v">{status}</span>
    </div>
  </button>

  <button class="prep glass" class:on={phase === "prepare"} onclick={() => setSleepPhase(phase === "prepare" ? "awake" : "prepare")}>
    <svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true">
      <path fill="currentColor" d="M12.7 2a8 8 0 1 0 9.3 9.3 7 7 0 0 1-9.3-9.3Z" />
    </svg>
    Prepare to sleep
  </button>

  <div class="grid">
    <GlassCard title="VRChat">
      <div class="stat"><span class="n">{world ? players : "—"}</span><span class="l">players in world</span></div>
      <div class="sub">{world ?? "Not in a world"}</div>
    </GlassCard>
    <GlassCard title="Headset">
      <div class="stat"><span class="n small">{backend ?? "—"}</span><span class="l">brightness backend</span></div>
      <div class="sub">{app.state?.overlay_running ? "Overlay running" : "Overlay not running"}</div>
    </GlassCard>
  </div>
</div>

<style>
  .view {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 880px;
    margin: 0 auto;
  }
  .sleepcard {
    display: flex;
    align-items: center;
    gap: 26px;
    width: 100%;
    padding: 30px 34px;
    border-radius: var(--radius-l);
    color: hsl(var(--muted-foreground));
    transition: all 0.2s var(--ease);
  }
  .sleepcard:hover {
    border-color: hsl(var(--glass-border) / 0.16);
  }
  .sleepcard.sleep {
    background: linear-gradient(135deg, hsl(var(--primary) / 0.9), hsl(258 60% 52% / 0.9));
    color: #fff;
    box-shadow: 0 14px 40px hsl(var(--primary) / 0.35);
  }
  .sleepcard.prepare {
    background: linear-gradient(135deg, hsl(212 55% 45% / 0.85), hsl(230 50% 40% / 0.85));
    color: #fff;
  }
  .sleepcard .txt {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    margin-left: auto;
    text-align: right;
  }
  .sleepcard .k {
    font-size: 16px;
    opacity: 0.85;
  }
  .sleepcard .v {
    font-size: 40px;
    font-weight: 700;
    color: hsl(var(--foreground));
    line-height: 1.1;
  }
  .sleepcard.sleep .v,
  .sleepcard.prepare .v {
    color: #fff;
  }
  .prep {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    width: 100%;
    height: 52px;
    border-radius: var(--radius-m);
    color: hsl(var(--muted-foreground));
    font-size: 15px;
    font-weight: 600;
    transition: all 0.15s var(--ease);
  }
  .prep:hover {
    color: hsl(var(--foreground));
    border-color: hsl(var(--glass-border) / 0.16);
  }
  .prep.on {
    background: hsl(212 55% 45% / 0.55);
    color: #fff;
  }
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 18px;
    margin-top: 2px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .stat .n {
    font-size: 34px;
    font-weight: 700;
  }
  .stat .n.small {
    font-size: 20px;
  }
  .stat .l {
    color: hsl(var(--muted-foreground));
    font-size: 13px;
  }
  .sub {
    margin-top: 12px;
    color: hsl(var(--muted-foreground));
    font-size: 13px;
  }
</style>
