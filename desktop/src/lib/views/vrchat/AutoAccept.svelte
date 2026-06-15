<script lang="ts">
  import { onMount } from "svelte";
  import { app, save, saveSoon, loadVrchatFriends } from "$lib/state.svelte";
  import { vrchatInviteMessages, vrchatUpdateInviteMessage } from "$lib/api";
  import type { InviteMessage } from "$lib/types";
  import GlassCard from "$lib/components/GlassCard.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
  import Slider from "$lib/components/Slider.svelte";
  import FriendPicker from "$lib/components/FriendPicker.svelte";

  let messages = $state<InviteMessage[]>([]);
  let loadingMsgs = $state(false);
  let msgError = $state<string | null>(null);
  let editText = $state("");
  let saving = $state(false);
  let saveNote = $state<string | null>(null);

  const selected = $derived(
    messages.find((m) => m.slot === app.config?.vrchat.auto_accept.invite_message_slot),
  );

  async function loadMessages() {
    loadingMsgs = true;
    msgError = null;
    try {
      messages = await vrchatInviteMessages();
      syncEditText();
    } catch (e) {
      msgError = String(e);
    } finally {
      loadingMsgs = false;
    }
  }

  function syncEditText() {
    editText = selected?.message ?? "";
    saveNote = null;
  }

  function onToggleMessages() {
    save();
    if (app.config?.vrchat.auto_accept.invite_message_enabled && messages.length === 0) loadMessages();
  }

  function onSlotChange() {
    save();
    syncEditText();
  }

  async function updateMessage() {
    if (!app.config) return;
    saving = true;
    saveNote = null;
    try {
      messages = await vrchatUpdateInviteMessage(app.config.vrchat.auto_accept.invite_message_slot, editText);
      saveNote = "Saved to VRChat.";
    } catch (e) {
      saveNote = String(e);
    } finally {
      saving = false;
    }
  }

  onMount(() => {
    if (app.vrchatLogin.logged_in && app.config?.vrchat.auto_accept.invite_message_enabled) loadMessages();
  });
</script>

<GlassCard
  title="Auto-Accept Invites"
  desc="Automatically accept invite requests in real time, limited to chosen friends and your world's player count."
>
  {#if !app.vrchatLogin.logged_in}
    <p class="msg">Sign in under <strong>Settings → VRChat Account</strong> to use auto-accept.</p>
  {:else if app.config}
    {@const aa = app.config.vrchat.auto_accept}
    <div class="rows">
      <div class="row">
        <div class="txt"><span class="t">Enable</span></div>
        <Toggle bind:checked={aa.enabled} label="Auto-accept" onchange={save} />
      </div>

      <div class="row">
        <div class="txt">
          <span class="t">List mode</span>
          <span class="d">
            {aa.list_mode === "whitelist"
              ? "Accept only from listed friends"
              : "Accept from everyone except listed friends"}
          </span>
        </div>
        <select bind:value={aa.list_mode} onchange={save}>
          <option value="whitelist">Whitelist</option>
          <option value="blacklist">Blacklist</option>
        </select>
      </div>

      <div class="row">
        <div class="txt"><span class="t">Only when sleep mode is enabled</span></div>
        <Toggle bind:checked={aa.only_when_sleep} label="Only when sleeping" onchange={save} />
      </div>

      <div class="row">
        <div class="txt">
          <span class="t">Only when fewer than N players</span>
          <span class="d">Counts everyone in your world, including you</span>
        </div>
        <Toggle bind:checked={aa.max_players_enabled} label="Limit by players" onchange={save} />
      </div>
      {#if aa.max_players_enabled}
        <div class="row slider">
          <Slider label="Max players" min={1} max={40} editable bind:value={aa.max_players} onchange={saveSoon} />
        </div>
      {/if}

      <div class="row">
        <div class="txt">
          <span class="t">Send an invite message</span>
          <span class="d">Attach one of your VRChat invite messages when accepting</span>
        </div>
        <Toggle bind:checked={aa.invite_message_enabled} label="Send invite message" onchange={onToggleMessages} />
      </div>
    </div>

    {#if aa.invite_message_enabled}
      <div class="msgbox">
        {#if loadingMsgs}
          <p class="msg">Loading your invite messages…</p>
        {:else if msgError}
          <p class="msg err">{msgError} <button class="link" onclick={loadMessages}>Retry</button></p>
        {:else if messages.length}
          <label class="field">
            <span class="lbl">Message slot</span>
            <select bind:value={aa.invite_message_slot} onchange={onSlotChange}>
              {#each messages as m (m.slot)}
                <option value={m.slot}>Slot {m.slot + 1} — {m.message || "(empty)"}</option>
              {/each}
            </select>
          </label>

          <label class="field">
            <span class="lbl">Message text</span>
            <textarea bind:value={editText} rows="2" maxlength="64" placeholder="e.g. Sleeping — come on in! zzz"></textarea>
          </label>

          <div class="msg-actions">
            <button
              class="btn tonal state-layer"
              disabled={saving || selected?.can_update === false || editText === (selected?.message ?? "")}
              onclick={updateMessage}
            >
              {saving ? "Saving…" : "Save message to VRChat"}
            </button>
            {#if selected && selected.can_update === false}
              <span class="cooldown">Editable again in ~{selected.cooldown_minutes} min</span>
            {:else if saveNote}
              <span class="cooldown">{saveNote}</span>
            {/if}
          </div>
          <p class="hint">VRChat limits message edits to about once per hour per slot.</p>
        {:else}
          <p class="msg">No invite messages found. <button class="link" onclick={loadMessages}>Reload</button></p>
        {/if}
      </div>
    {/if}

    <div class="list-head">
      <strong>{aa.list_mode === "whitelist" ? "Whitelist" : "Blacklist"}</strong>
      <span class="count">{aa.player_ids.length} added</span>
    </div>
    <FriendPicker bind:ids={aa.player_ids} friends={app.vrchatFriends} onLoad={loadVrchatFriends} onchange={save} />
  {/if}
</GlassCard>

<style>
  .msg {
    margin: 0;
    color: hsl(var(--muted-foreground));
    font-size: 13.5px;
    line-height: 1.5;
  }
  .msg strong {
    color: hsl(var(--foreground));
  }
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
  .row.slider {
    display: block;
    padding-top: 4px;
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
  .msgbox {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-top: 14px;
    padding-top: 14px;
    border-top: 1px solid hsl(var(--glass-border) / 0.06);
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
  textarea {
    border-radius: var(--radius-s);
    border: 1px solid hsl(var(--glass-border) / 0.1);
    background: hsl(var(--glass-bg) / 0.5);
    color: hsl(var(--foreground));
    padding: 10px 12px;
    font-size: 14px;
    font-family: inherit;
    resize: vertical;
  }
  textarea:focus {
    outline: none;
    border-color: hsl(var(--primary) / 0.6);
  }
  .msg-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .msg-actions .btn {
    height: 34px;
    padding: 0 16px;
    font-size: 13px;
  }
  .msg-actions .btn:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .cooldown {
    font-size: 12.5px;
    color: hsl(var(--muted-foreground));
  }
  .hint {
    margin: 0;
    font-size: 12px;
    color: hsl(var(--muted-foreground));
  }
  .msg.err {
    color: hsl(var(--destructive, 0 70% 65%));
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
  .list-head {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 20px 0 12px;
  }
  .list-head strong {
    font-size: 14.5px;
  }
  .list-head .count {
    color: hsl(var(--muted-foreground));
    font-size: 13px;
  }
</style>
