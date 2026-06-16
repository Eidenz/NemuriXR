<script lang="ts">
  import { app } from "$lib/state.svelte";
  import WindowControls from "./WindowControls.svelte";

  interface Tab {
    id: string;
    label: string;
  }
  interface Props {
    active: string;
    tabs: Tab[];
    onChange: (id: string) => void;
    connected: boolean;
  }
  let { active, tabs, onChange, connected }: Props = $props();
</script>

<header class="bar glass-strong" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
      <path
        fill="currentColor"
        d="M12.7 2a8 8 0 1 0 9.3 9.3 7 7 0 0 1-9.3-9.3Z"
      />
    </svg>
    <span class="name">Nemuri<b>XR</b></span>
    {#if app.version}<span class="ver">v{app.version}</span>{/if}
  </div>

  <nav class="tabs">
    {#each tabs as t (t.id)}
      <button class="tab state-layer" class:active={active === t.id} onclick={() => onChange(t.id)}>
        {t.label}
      </button>
    {/each}
  </nav>

  <div class="right">
    <div class="chip">
      <span class="dot" class:on={connected} class:warn={!connected}></span>
      {connected ? "VR overlay connected" : "VR overlay offline"}
    </div>
    <WindowControls />
  </div>
</header>

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 10px 8px 18px;
    height: 56px;
    flex: none;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 9px;
    color: hsl(var(--primary));
    padding-right: 6px;
  }
  .name {
    font-size: 18px;
    font-weight: 700;
    color: hsl(var(--foreground));
    letter-spacing: 0.2px;
  }
  .name b {
    color: hsl(var(--primary));
  }
  .ver {
    font-size: 11px;
    font-weight: 600;
    color: hsl(var(--muted-foreground));
    align-self: flex-end;
    padding-bottom: 2px;
  }
  .tabs {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: 1;
  }
  .tab {
    height: 36px;
    padding: 0 16px;
    border-radius: var(--radius-pill);
    font-size: 14px;
    font-weight: 600;
    color: hsl(var(--muted-foreground));
    transition: color 0.15s var(--ease), background 0.15s var(--ease);
  }
  .tab:hover {
    color: hsl(var(--foreground));
  }
  .tab.active {
    color: hsl(var(--foreground));
    background: hsl(var(--primary) / 0.22);
  }
  .right {
    display: flex;
    align-items: center;
    gap: 12px;
  }
</style>
