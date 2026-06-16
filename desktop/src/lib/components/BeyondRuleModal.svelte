<script lang="ts">
  import { app, checkBeyond } from "$lib/state.svelte";
  import { installBeyondRule } from "$lib/api";

  const DISMISS_KEY = "nemurixr.beyondDismissed";
  const ls = typeof localStorage !== "undefined" ? localStorage : null;

  let dismissed = $state(ls?.getItem(DISMISS_KEY) === "1");
  let installing = $state(false);
  let error = $state<string | null>(null);

  // Show only when a Beyond is connected but we can't talk to its HID yet.
  const open = $derived(app.beyondStatus === "needs_rule" && !dismissed);

  async function install() {
    installing = true;
    error = null;
    try {
      await installBeyondRule();
      await checkBeyond();
      if (app.beyondStatus !== "ready") {
        error = "Installed, but no access yet — try replugging the headset.";
      }
    } catch (e) {
      error = String(e);
    } finally {
      installing = false;
    }
  }

  function notNow() {
    dismissed = true;
    ls?.setItem(DISMISS_KEY, "1");
  }
</script>

{#if open}
  <div class="scrim">
    <div class="modal glass-strong">
      <h2>Bigscreen Beyond detected</h2>
      <p>
        To control its brightness and fans, NemuriXR needs a small udev rule that grants access to the headset.
        Install it now? You'll be asked to authorize the change.
      </p>
      {#if error}
        <p class="err">{error}</p>
      {/if}
      <div class="actions">
        <button class="btn ghost state-layer" disabled={installing} onclick={notNow}>Not now</button>
        <button class="btn primary state-layer" disabled={installing} onclick={install}>
          {installing ? "Installing…" : "Install rule"}
        </button>
      </div>
      <p class="hint">You can also do this later from <strong>Settings → General</strong>.</p>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    background: hsl(0 0% 0% / 0.5);
    backdrop-filter: blur(2px);
  }
  .modal {
    width: min(440px, calc(100vw - 48px));
    padding: 24px;
    border-radius: var(--radius-l);
  }
  h2 {
    font-size: 19px;
    margin: 0 0 10px;
  }
  p {
    margin: 0 0 12px;
    color: hsl(var(--muted-foreground));
    font-size: 13.5px;
    line-height: 1.5;
  }
  .err {
    color: hsl(0 70% 68%);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 18px;
  }
  .btn {
    height: 38px;
    padding: 0 18px;
    border-radius: var(--radius-m);
    font-size: 14px;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .btn.primary {
    background: hsl(var(--primary));
    color: #fff;
    border: none;
  }
  .btn.ghost {
    background: transparent;
    color: hsl(var(--foreground));
    border: 1px solid hsl(var(--glass-border) / 0.16);
  }
  .hint {
    margin: 14px 0 0;
    font-size: 12px;
  }
  .hint strong {
    color: hsl(var(--foreground));
  }
</style>
