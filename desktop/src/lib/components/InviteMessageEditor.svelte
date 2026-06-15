<script lang="ts">
  import { onMount } from "svelte";
  import { vrchatMessages, vrchatUpdateMessage, type MessageKind } from "$lib/api";
  import type { InviteMessage } from "$lib/types";
  import Toggle from "./Toggle.svelte";

  interface Props {
    kind: MessageKind;
    title: string;
    desc: string;
    enabled: boolean;
    slot: number;
    onchange?: () => void;
    placeholder?: string;
  }
  let {
    kind,
    title,
    desc,
    enabled = $bindable(false),
    slot = $bindable(0),
    onchange,
    placeholder = "",
  }: Props = $props();

  let messages = $state<InviteMessage[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let editText = $state("");
  let saving = $state(false);
  let note = $state<string | null>(null);

  const selected = $derived(messages.find((m) => m.slot === slot));

  async function load() {
    loading = true;
    error = null;
    try {
      messages = await vrchatMessages(kind);
      syncText();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
  function syncText() {
    editText = selected?.message ?? "";
    note = null;
  }
  function onToggle() {
    onchange?.();
    if (enabled && messages.length === 0) load();
  }
  function onSlot() {
    onchange?.();
    syncText();
  }
  async function saveText() {
    saving = true;
    note = null;
    try {
      messages = await vrchatUpdateMessage(kind, slot, editText);
      note = "Saved to VRChat.";
    } catch (e) {
      note = String(e);
    } finally {
      saving = false;
    }
  }
  onMount(() => {
    if (enabled) load();
  });
</script>

<div class="editor">
  <div class="row">
    <div class="txt">
      <span class="t">{title}</span>
      <span class="d">{desc}</span>
    </div>
    <Toggle bind:checked={enabled} label={title} onchange={onToggle} />
  </div>

  {#if enabled}
    <div class="body">
      {#if loading}
        <p class="m">Loading messages…</p>
      {:else if error}
        <p class="m err">{error} <button class="link" onclick={load}>Retry</button></p>
      {:else if messages.length}
        <label class="field">
          <span class="lbl">Message slot</span>
          <select bind:value={slot} onchange={onSlot}>
            {#each messages as m (m.slot)}
              <option value={m.slot}>Slot {m.slot + 1} — {m.message || "(empty)"}</option>
            {/each}
          </select>
        </label>
        <label class="field">
          <span class="lbl">Message text</span>
          <textarea bind:value={editText} rows="2" maxlength="64" {placeholder}></textarea>
        </label>
        <div class="actions">
          <button
            class="btn tonal state-layer"
            disabled={saving || selected?.can_update === false || editText === (selected?.message ?? "")}
            onclick={saveText}
          >
            {saving ? "Saving…" : "Save to VRChat"}
          </button>
          {#if selected && selected.can_update === false}
            <span class="cd">Editable again in ~{selected.cooldown_minutes} min</span>
          {:else if note}
            <span class="cd">{note}</span>
          {/if}
        </div>
        <p class="hint">VRChat limits message edits to about once per hour per slot.</p>
      {:else}
        <p class="m">No messages found. <button class="link" onclick={load}>Reload</button></p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .editor {
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
  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 2px 0 14px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .field .lbl {
    font-size: 12.5px;
    font-weight: 600;
    color: hsl(var(--muted-foreground));
  }
  select,
  textarea {
    border-radius: var(--radius-s);
    border: 1px solid hsl(var(--glass-border) / 0.1);
    background: hsl(var(--glass-bg) / 0.5);
    color: hsl(var(--foreground));
    padding: 10px 12px;
    font-size: 14px;
    font-family: inherit;
  }
  select {
    height: 36px;
    padding: 0 12px;
  }
  textarea {
    resize: vertical;
  }
  select:focus,
  textarea:focus {
    outline: none;
    border-color: hsl(var(--primary) / 0.6);
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .actions .btn {
    height: 34px;
    padding: 0 16px;
    font-size: 13px;
  }
  .actions .btn:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .cd {
    font-size: 12.5px;
    color: hsl(var(--muted-foreground));
  }
  .hint {
    margin: 0;
    font-size: 12px;
    color: hsl(var(--muted-foreground));
  }
  .m {
    margin: 0;
    color: hsl(var(--muted-foreground));
    font-size: 13.5px;
  }
  .m.err {
    color: hsl(0 70% 68%);
  }
  .link {
    background: none;
    border: none;
    color: hsl(var(--primary));
    cursor: pointer;
    font: inherit;
    padding: 0;
    text-decoration: underline;
  }
</style>
