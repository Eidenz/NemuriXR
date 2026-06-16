<script lang="ts">
  import { app, save, saveSoon } from "$lib/state.svelte";
  import { testSleepingPose } from "$lib/api";
  import { POSE_PRESETS } from "$lib/sleepingPosePresets";
  import type { OscMessage } from "$lib/types";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
  import OscRow from "$lib/components/OscRow.svelte";

  const positions = [
    { key: "on_back", title: "On your back" },
    { key: "on_front", title: "On your front" },
    { key: "on_left", title: "On your left side" },
    { key: "on_right", title: "On your right side" },
  ] as const;

  function applyPreset(id: string) {
    const sp = app.config?.vrchat.sleeping_pose;
    if (!sp) return;
    if (id === "custom") {
      sp.preset = "custom";
      save();
      return;
    }
    const p = POSE_PRESETS.find((x) => x.id === id);
    if (!p) return;
    sp.preset = id;
    sp.lock_feet = p.lock_feet;
    sp.on_back = structuredClone(p.on_back);
    sp.on_front = structuredClone(p.on_front);
    sp.on_left = structuredClone(p.on_left);
    sp.on_right = structuredClone(p.on_right);
    sp.foot_lock = structuredClone(p.foot_lock);
    sp.foot_unlock = structuredClone(p.foot_unlock);
    save();
  }

  function addMsg(list: OscMessage[]) {
    list.push({ address: "/avatar/parameters/", args: [{ kind: "bool", value: true }], delay_ms: 0 });
    save();
  }
  function removeMsg(list: OscMessage[], i: number) {
    list.splice(i, 1);
    save();
  }
</script>

{#if app.config}
  {@const sp = app.config.vrchat.sleeping_pose}
  <GlassCard
    title="Sleeping Pose"
    desc="While you sleep, lie your avatar in the direction you're physically lying (detected from your headset). Sends OSC, so it needs VRChat OSC on — it reuses the connection from the OSC tab. No login required."
  >
    <div class="rows">
      <div class="row">
        <div class="txt">
          <span class="t">Pose continuously while asleep</span>
          <span class="d">The pose set below is also used by the Safety Net — configure it even with this off</span>
        </div>
        <Toggle bind:checked={sp.enabled} label="Sleeping pose" onchange={save} />
      </div>

      <div class="row">
        <div class="txt">
          <span class="t">Avatar system</span>
          <span class="d">Presets fill in the OSC; pick Custom to edit it yourself</span>
        </div>
        <select value={sp.preset} onchange={(e) => applyPreset(e.currentTarget.value)}>
          <option value="" disabled>Choose…</option>
          {#each POSE_PRESETS as p (p.id)}<option value={p.id}>{p.label}</option>{/each}
          <option value="custom">Custom</option>
        </select>
      </div>

      <div class="row">
        <div class="txt">
          <span class="t">Lock feet (stop sliding)</span>
          <span class="d">Briefly releases and re-locks on each pose change</span>
        </div>
        <Toggle bind:checked={sp.lock_feet} label="Lock feet" onchange={save} />
      </div>

      <div class="row" class:dim={!sp.preset}>
        <div class="txt"><span class="t">Test a pose</span><span class="d">Send it to your avatar now</span></div>
        <div class="tests">
          <button class="btn tonal state-layer" onclick={() => testSleepingPose("back")}>Back</button>
          <button class="btn tonal state-layer" onclick={() => testSleepingPose("front")}>Front</button>
          <button class="btn tonal state-layer" onclick={() => testSleepingPose("left")}>Left</button>
          <button class="btn tonal state-layer" onclick={() => testSleepingPose("right")}>Right</button>
        </div>
      </div>
    </div>
  </GlassCard>

  {#if sp.preset === "custom"}
    {#each positions as pos (pos.key)}
      <GlassCard title={pos.title}>
        <div class="msgs">
          {#each sp[pos.key] as msg, i (i)}
            <OscRow message={msg} onchange={saveSoon} onremove={() => removeMsg(sp[pos.key], i)} />
          {/each}
          {#if sp[pos.key].length === 0}
            <p class="empty">No messages — nothing is sent for this position.</p>
          {/if}
        </div>
        <button class="btn tonal state-layer add" onclick={() => addMsg(sp[pos.key])}>+ Add message</button>
      </GlassCard>
    {/each}

    <GlassCard title="Feet (anti-slide)">
      <p class="lede">Sent when locking / unlocking the feet (if “Lock feet” is on).</p>
      <div class="feet">
        <div>
          <span class="flbl">Lock</span>
          {#each sp.foot_lock as msg, i (i)}
            <OscRow message={msg} onchange={saveSoon} onremove={() => removeMsg(sp.foot_lock, i)} />
          {/each}
          <button class="btn tonal state-layer add" onclick={() => addMsg(sp.foot_lock)}>+ Add</button>
        </div>
        <div>
          <span class="flbl">Unlock</span>
          {#each sp.foot_unlock as msg, i (i)}
            <OscRow message={msg} onchange={saveSoon} onremove={() => removeMsg(sp.foot_unlock, i)} />
          {/each}
          <button class="btn tonal state-layer add" onclick={() => addMsg(sp.foot_unlock)}>+ Add</button>
        </div>
      </div>
    </GlassCard>
  {:else if sp.preset}
    <p class="note">
      Using the <strong>{POSE_PRESETS.find((p) => p.id === sp.preset)?.label ?? sp.preset}</strong> preset — it sends the
      right avatar parameters for each side automatically. Switch to <strong>Custom</strong> above to tweak the OSC.
    </p>
  {/if}
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
  .tests {
    display: flex;
    gap: 6px;
  }
  .tests .btn {
    height: 34px;
    padding: 0 12px;
    font-size: 13px;
  }
  .msgs {
    display: flex;
    flex-direction: column;
  }
  .empty {
    color: hsl(var(--muted-foreground));
    font-size: 13px;
    margin: 4px 0;
  }
  .add {
    margin-top: 12px;
    height: 34px;
    padding: 0 14px;
    font-size: 13px;
  }
  .lede {
    margin: 0 0 12px;
    color: hsl(var(--muted-foreground));
    font-size: 13px;
  }
  .feet {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .flbl {
    display: block;
    font-weight: 600;
    font-size: 13px;
    margin-bottom: 6px;
  }
  .note {
    margin: 0;
    color: hsl(var(--muted-foreground));
    font-size: 13px;
    line-height: 1.5;
    background: hsl(var(--primary) / 0.1);
    border: 1px solid hsl(var(--primary) / 0.2);
    border-radius: var(--radius-m);
    padding: 12px 16px;
    max-width: 880px;
  }
  .note strong {
    color: hsl(var(--foreground));
  }
</style>
