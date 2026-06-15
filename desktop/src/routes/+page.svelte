<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { app, startPolling, stopPolling } from "$lib/state.svelte";
  import Backdrop from "$lib/components/Backdrop.svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import VrchatLive from "$lib/components/VrchatLive.svelte";
  import StatusView from "$lib/views/StatusView.svelte";
  import BrightnessView from "$lib/views/BrightnessView.svelte";
  import AudioView from "$lib/views/AudioView.svelte";
  import OscView from "$lib/views/OscView.svelte";
  import JoinNotifications from "$lib/views/vrchat/JoinNotifications.svelte";
  import AutoAccept from "$lib/views/vrchat/AutoAccept.svelte";
  import StatusAutomations from "$lib/views/vrchat/StatusAutomations.svelte";
  import Account from "$lib/views/settings/Account.svelte";
  import Schedule from "$lib/views/settings/Schedule.svelte";
  import SleepDetection from "$lib/views/settings/SleepDetection.svelte";
  import Commands from "$lib/views/settings/Commands.svelte";
  import General from "$lib/views/settings/General.svelte";

  interface Section {
    id: string;
    label: string;
  }
  interface Tab {
    id: string;
    label: string;
    sections?: Section[];
  }
  const tabs: Tab[] = [
    { id: "status", label: "Status" },
    { id: "brightness", label: "Brightness" },
    { id: "audio", label: "Audio" },
    {
      id: "vrchat",
      label: "VRChat",
      sections: [
        { id: "join", label: "Join Notifications" },
        { id: "autoaccept", label: "Auto-Accept" },
        { id: "statusauto", label: "Status Automations" },
      ],
    },
    { id: "osc", label: "OSC" },
    {
      id: "settings",
      label: "Settings",
      sections: [
        { id: "general", label: "General" },
        { id: "schedule", label: "Sleep Schedule" },
        { id: "detection", label: "Sleep Detection" },
        { id: "commands", label: "Commands" },
        { id: "account", label: "VRChat Account" },
      ],
    },
  ];

  let active = $state("status");
  // Remembers the chosen sub-section per top tab.
  let section = $state<Record<string, string>>({ vrchat: "join", settings: "general" });

  const current = $derived(tabs.find((t) => t.id === active) ?? tabs[0]);
  const sections = $derived(current.sections ?? []);
  const sub = $derived(section[active] ?? sections[0]?.id ?? "");

  onMount(startPolling);
  onDestroy(stopPolling);
</script>

<Backdrop />
<div class="app">
  <TitleBar
    {active}
    tabs={tabs.map((t) => ({ id: t.id, label: t.label }))}
    onChange={(id) => (active = id)}
    connected={app.state?.overlay_running ?? false}
  />
  <main class="main">
    {#if sections.length}
      <aside class="subnav glass">
        {#each sections as s (s.id)}
          <button class="subitem state-layer" class:active={sub === s.id} onclick={() => (section[active] = s.id)}>
            {s.label}
          </button>
        {/each}
      </aside>
      <div class="content">
        <div class="section">
          {#if active === "vrchat"}
            <VrchatLive />
            {#if sub === "join"}
              <JoinNotifications />
            {:else if sub === "autoaccept"}
              <AutoAccept />
            {:else}
              <StatusAutomations />
            {/if}
          {:else if active === "settings"}
            {#if sub === "account"}
              <Account />
            {:else if sub === "schedule"}
              <Schedule />
            {:else if sub === "detection"}
              <SleepDetection />
            {:else if sub === "commands"}
              <Commands />
            {:else}
              <General />
            {/if}
          {/if}
        </div>
      </div>
    {:else}
      <div class="content">
        {#if active === "status"}
          <StatusView />
        {:else if active === "brightness"}
          <BrightnessView />
        {:else if active === "audio"}
          <AudioView />
        {:else}
          <OscView />
        {/if}
      </div>
    {/if}
  </main>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .main {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  .subnav {
    width: 212px;
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 14px 10px;
    margin: 16px 0 16px 16px;
    border-radius: var(--radius-l);
    align-self: flex-start;
  }
  .subitem {
    text-align: left;
    padding: 10px 14px;
    border-radius: var(--radius-m);
    color: hsl(var(--muted-foreground));
    font-size: 14px;
    font-weight: 600;
    transition: color 0.15s var(--ease), background 0.15s var(--ease);
  }
  .subitem:hover {
    color: hsl(var(--foreground));
  }
  .subitem.active {
    color: hsl(var(--foreground));
    background: hsl(var(--primary) / 0.22);
  }
  .content {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: 26px 28px 32px;
  }
  .section {
    max-width: 760px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
</style>
