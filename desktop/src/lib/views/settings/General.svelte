<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { app, save, checkBeyond, checkForUpdate } from "$lib/state.svelte";
  import { launchOverlay, installBeyondRule, beyondRuleText } from "$lib/api";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";

  let launching = $state(false);
  let result = $state<"" | "ok" | "fail">("");

  async function launch() {
    launching = true;
    result = "";
    try {
      result = (await launchOverlay()) ? "ok" : "fail";
      setTimeout(() => (result = ""), 4000);
    } catch {
      result = "fail";
    } finally {
      launching = false;
    }
  }

  // --- Bigscreen Beyond udev rule ---
  let installing = $state(false);
  let beyondError = $state<string | null>(null);
  let showRule = $state(false);
  let ruleText = $state("");

  async function installRule() {
    installing = true;
    beyondError = null;
    try {
      await installBeyondRule();
      await checkBeyond();
      if (app.beyondStatus !== "ready") beyondError = "Installed, but no access yet — try replugging the headset.";
    } catch (e) {
      beyondError = String(e);
    } finally {
      installing = false;
    }
  }

  async function toggleRule() {
    showRule = !showRule;
    if (showRule && !ruleText) ruleText = await beyondRuleText();
  }

  // --- Updates ---
  let checking = $state(false);
  let checkedMsg = $state<string | null>(null);
  async function recheck() {
    checking = true;
    checkedMsg = null;
    await checkForUpdate();
    checking = false;
    if (!app.update) checkedMsg = "You're on the latest version.";
  }
  function viewRelease() {
    if (app.update?.url) openUrl(app.update.url);
  }

  onMount(checkBeyond);
</script>

{#if app.config}
  <GlassCard title="In-Headset Overlay">
    <div class="rows">
      <div class="row">
        <div class="txt">
          <span class="t">Auto-launch with VR</span>
          <span class="d">Start the in-headset overlay automatically when a Monado VR session begins (it closes itself when VR ends)</span>
        </div>
        <Toggle bind:checked={app.config.auto_launch_overlay} label="Auto-launch overlay" onchange={save} />
      </div>
      <div class="row">
        <div class="txt">
          <span class="t">Launch now</span>
          <span class="d">Start the overlay manually (it'll exit if no VR runtime is running)</span>
        </div>
        <button class="btn tonal state-layer" disabled={launching} onclick={launch}>
          {launching ? "Launching…" : result === "ok" ? "Launched ✓" : result === "fail" ? "Overlay not found" : "Launch VR overlay"}
        </button>
      </div>
    </div>
  </GlassCard>

  <!-- Only relevant when a Beyond is actually connected. -->
  {#if app.beyondStatus !== "absent"}
    <GlassCard title="Bigscreen Beyond">
      <div class="rows">
        <div class="row">
          <div class="txt">
            <span class="t">Brightness & fan access</span>
            <span class="d">
              {#if app.beyondStatus === "ready"}
                Ready — NemuriXR can control your Beyond.
              {:else}
                A Beyond is connected but needs a udev rule for brightness/fan control.
              {/if}
            </span>
          </div>
          {#if app.beyondStatus === "ready"}
            <span class="ok">Ready ✓</span>
          {:else}
            <button class="btn tonal state-layer" disabled={installing} onclick={installRule}>
              {installing ? "Installing…" : "Install rule"}
            </button>
          {/if}
        </div>
        {#if beyondError}
          <p class="note err">{beyondError}</p>
        {/if}
        <div class="row">
          <div class="txt">
            <span class="t">Manual install</span>
            <span class="d">Prefer to add the rule yourself? Show the udev rule to copy.</span>
          </div>
          <button class="btn tonal state-layer" onclick={toggleRule}>{showRule ? "Hide" : "Show rule"}</button>
        </div>
        {#if showRule}
          <pre class="rule">{ruleText}</pre>
          <p class="note">Save it to <code>/etc/udev/rules.d/70-nemurixr-beyond.rules</code>, then run
            <code>sudo udevadm control --reload-rules &amp;&amp; sudo udevadm trigger</code>.</p>
        {/if}
      </div>
    </GlassCard>
  {/if}

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

  <GlassCard title="About">
    <div class="rows">
      <div class="row">
        <div class="txt">
          <span class="t">NemuriXR v{app.version}</span>
          <span class="d">
            {#if app.update}
              Update available: v{app.update.version}
            {:else if checkedMsg}
              {checkedMsg}
            {:else}
              Check GitHub for a newer release.
            {/if}
          </span>
        </div>
        {#if app.update}
          <button class="btn tonal state-layer" onclick={viewRelease}>View release</button>
        {:else}
          <button class="btn tonal state-layer" disabled={checking} onclick={recheck}>
            {checking ? "Checking…" : "Check for updates"}
          </button>
        {/if}
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
    border-top: 1px solid hsl(var(--glass-border) / 0.06);
  }
  .row:first-child {
    border-top: none;
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
  .btn {
    height: 36px;
    padding: 0 16px;
    font-size: 13px;
    flex: none;
  }
  .ok {
    color: hsl(140 50% 62%);
    font-weight: 600;
    font-size: 13.5px;
    flex: none;
  }
  .note {
    margin: 2px 0 0;
    color: hsl(var(--muted-foreground));
    font-size: 12.5px;
    line-height: 1.5;
  }
  .note.err {
    color: hsl(0 70% 68%);
  }
  .note code {
    font-family: var(--font-mono, monospace);
    font-size: 11.5px;
  }
  .rule {
    margin: 4px 0 0;
    padding: 10px 12px;
    border-radius: var(--radius-s);
    background: hsl(var(--glass-bg) / 0.5);
    border: 1px solid hsl(var(--glass-border) / 0.1);
    font-family: var(--font-mono, monospace);
    font-size: 11.5px;
    white-space: pre-wrap;
    word-break: break-all;
    color: hsl(var(--foreground));
  }
</style>
