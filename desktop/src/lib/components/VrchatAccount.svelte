<script lang="ts">
  import { onMount } from "svelte";
  import { vrchatLogin, vrchatVerify2fa, vrchatLogout } from "$lib/api";
  import { app, loadVrchatLogin } from "$lib/state.svelte";
  import GlassCard from "./GlassCard.svelte";

  let username = $state("");
  let password = $state("");
  let busy = $state(false);
  let error = $state("");

  // 2FA step
  let methods = $state<string[] | null>(null);
  let method = $state("");
  let code = $state("");

  const methodLabel = (m: string) =>
    m === "totp" ? "Authenticator app" : m === "emailOtp" ? "Email code" : "One-time code";

  onMount(loadVrchatLogin);

  async function login() {
    if (busy) return;
    busy = true;
    error = "";
    try {
      const res = await vrchatLogin(username, password);
      if (res.kind === "logged_in") {
        app.vrchatLogin = { logged_in: true, username: res.username };
        password = "";
      } else if (res.kind === "needs_2fa") {
        methods = res.methods;
        method = res.methods[0] ?? "totp";
        password = "";
      } else {
        error = res.message;
      }
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function verify() {
    if (busy) return;
    busy = true;
    error = "";
    try {
      const res = await vrchatVerify2fa(method, code);
      if (res.kind === "logged_in") {
        app.vrchatLogin = { logged_in: true, username: res.username };
        methods = null;
        code = "";
      } else if (res.kind === "failed") {
        error = res.message;
      }
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function logout() {
    busy = true;
    try {
      app.vrchatLogin = await vrchatLogout();
      methods = null;
      username = "";
      code = "";
    } finally {
      busy = false;
    }
  }
</script>

<GlassCard title="VRChat Account" desc="Sign in to enable auto-accept invites and status automations. Your password is never stored — only a session token, kept in your system keyring.">
  {#if app.vrchatLogin.logged_in}
    <div class="loggedin">
      <div class="who">
        <span class="dot on"></span>
        Signed in as <strong>{app.vrchatLogin.username}</strong>
      </div>
      <button class="btn tonal state-layer" disabled={busy} onclick={logout}>Sign out</button>
    </div>
  {:else if methods}
    <form class="form" onsubmit={(e) => (e.preventDefault(), verify())}>
      <p class="hint">Enter the verification code ({methodLabel(method)}).</p>
      {#if methods.length > 1}
        <select bind:value={method}>
          {#each methods as m (m)}
            <option value={m}>{methodLabel(m)}</option>
          {/each}
        </select>
      {/if}
      <input type="text" inputmode="numeric" autocomplete="one-time-code" placeholder="123456" bind:value={code} />
      <div class="actions">
        <button class="btn filled state-layer" type="submit" disabled={busy || !code}>Verify</button>
        <button class="btn tonal state-layer" type="button" disabled={busy} onclick={() => (methods = null)}>Back</button>
      </div>
    </form>
  {:else}
    <form class="form" onsubmit={(e) => (e.preventDefault(), login())}>
      <input type="text" autocomplete="username" placeholder="VRChat username or email" bind:value={username} />
      <input type="password" autocomplete="current-password" placeholder="Password" bind:value={password} />
      <button class="btn filled state-layer" type="submit" disabled={busy || !username || !password}>
        {busy ? "Signing in…" : "Sign in"}
      </button>
    </form>
  {/if}

  {#if error}
    <p class="err">{error}</p>
  {/if}
</GlassCard>

<style>
  .loggedin {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }
  .who {
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: 14.5px;
    color: hsl(var(--muted-foreground));
  }
  .who strong {
    color: hsl(var(--foreground));
  }
  .form {
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 360px;
  }
  .hint {
    margin: 0;
    color: hsl(var(--muted-foreground));
    font-size: 13px;
  }
  input,
  select {
    height: 40px;
    border-radius: var(--radius-s);
    border: 1px solid hsl(var(--glass-border) / 0.1);
    background: hsl(var(--glass-bg) / 0.5);
    color: hsl(var(--foreground));
    padding: 0 12px;
    font-size: 14px;
    font-family: inherit;
  }
  input:focus,
  select:focus {
    outline: none;
    border-color: hsl(var(--primary) / 0.6);
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .err {
    margin: 12px 0 0;
    color: hsl(var(--error));
    font-size: 13px;
  }
</style>
