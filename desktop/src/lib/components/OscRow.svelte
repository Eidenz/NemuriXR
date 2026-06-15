<script lang="ts">
  import type { OscArg, OscMessage } from "$lib/types";

  interface Props {
    message: OscMessage;
    onchange?: () => void;
    onremove?: () => void;
  }
  let { message, onchange, onremove }: Props = $props();
  type Kind = OscArg["kind"];

  // Single typed value per message (the common VRChat case); fall back to a
  // bool if a loaded message somehow has no args (writes fix it on first edit).
  const arg = $derived(message.args[0] ?? ({ kind: "bool", value: true } as OscArg));

  function setKind(kind: Kind) {
    const value = kind === "string" ? "" : kind === "bool" ? true : 0;
    message.args = [{ kind, value } as OscArg];
    onchange?.();
  }
  function setValue(value: boolean | number | string) {
    message.args = [{ kind: arg.kind, value } as OscArg];
    onchange?.();
  }
  function setDelay(v: number) {
    message.delay_ms = Math.max(0, Math.round(v || 0));
    onchange?.();
  }
</script>

<div class="row">
  <input
    class="addr"
    type="text"
    placeholder="/avatar/parameters/…"
    bind:value={message.address}
    oninput={() => onchange?.()}
  />

  <select class="kind" value={arg.kind} onchange={(e) => setKind(e.currentTarget.value as Kind)}>
    <option value="bool">bool</option>
    <option value="int">int</option>
    <option value="float">float</option>
    <option value="string">string</option>
  </select>

  <div class="val">
    {#if arg.kind === "bool"}
      <label class="boolval">
        <input type="checkbox" checked={arg.value} onchange={(e) => setValue(e.currentTarget.checked)} />
        {arg.value ? "true" : "false"}
      </label>
    {:else if arg.kind === "int"}
      <input type="number" step="1" value={arg.value} oninput={(e) => setValue(Math.round(Number(e.currentTarget.value)))} />
    {:else if arg.kind === "float"}
      <input type="number" step="0.1" value={arg.value} oninput={(e) => setValue(Number(e.currentTarget.value))} />
    {:else}
      <input type="text" value={arg.value} oninput={(e) => setValue(e.currentTarget.value)} />
    {/if}
  </div>

  <input class="delay" type="number" min="0" step="50" value={message.delay_ms} title="Delay before sending (ms)" oninput={(e) => setDelay(Number(e.currentTarget.value))} />
  <span class="ms">ms</span>

  <button class="rm" aria-label="Remove message" onclick={() => onremove?.()}>✕</button>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 0;
  }
  input,
  select {
    height: 34px;
    border-radius: var(--radius-s);
    border: 1px solid hsl(var(--glass-border) / 0.1);
    background: hsl(var(--glass-bg) / 0.5);
    color: hsl(var(--foreground));
    padding: 0 10px;
    font-size: 13px;
    font-family: inherit;
  }
  input:focus,
  select:focus {
    outline: none;
    border-color: hsl(var(--primary) / 0.6);
  }
  .addr {
    flex: 1;
    min-width: 0;
  }
  .kind {
    width: 84px;
  }
  .val {
    width: 130px;
    display: flex;
  }
  .val input {
    width: 100%;
  }
  .boolval {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: hsl(var(--muted-foreground));
  }
  .boolval input {
    width: 18px;
    height: 18px;
    padding: 0;
    accent-color: hsl(var(--primary));
  }
  .delay {
    width: 72px;
  }
  .ms {
    color: hsl(var(--muted-foreground));
    font-size: 12px;
    margin-left: -2px;
  }
  .rm {
    width: 30px;
    height: 30px;
    border-radius: var(--radius-s);
    color: hsl(var(--muted-foreground));
    flex: none;
  }
  .rm:hover {
    background: hsl(var(--error) / 0.18);
    color: hsl(var(--error));
  }
</style>
