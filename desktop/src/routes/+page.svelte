<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { app, startPolling, stopPolling } from "$lib/state.svelte";
  import Backdrop from "$lib/components/Backdrop.svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import StatusView from "$lib/views/StatusView.svelte";
  import BrightnessView from "$lib/views/BrightnessView.svelte";
  import VrchatView from "$lib/views/VrchatView.svelte";
  import OscView from "$lib/views/OscView.svelte";
  import SettingsView from "$lib/views/SettingsView.svelte";

  const tabs = [
    { id: "status", label: "Status" },
    { id: "brightness", label: "Brightness" },
    { id: "vrchat", label: "VRChat" },
    { id: "osc", label: "OSC" },
    { id: "settings", label: "Settings" },
  ];
  let active = $state("status");

  onMount(startPolling);
  onDestroy(stopPolling);
</script>

<Backdrop />
<div class="app">
  <TitleBar {active} {tabs} onChange={(id) => (active = id)} connected={app.state?.overlay_running ?? false} />
  <main class="content">
    {#if active === "status"}
      <StatusView />
    {:else if active === "brightness"}
      <BrightnessView />
    {:else if active === "vrchat"}
      <VrchatView />
    {:else if active === "osc"}
      <OscView />
    {:else}
      <SettingsView />
    {/if}
  </main>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 26px 28px 32px;
  }
</style>
