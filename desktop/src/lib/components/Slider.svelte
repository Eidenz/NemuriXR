<script lang="ts">
  interface Props {
    value?: number;
    min?: number;
    max?: number;
    step?: number;
    suffix?: string;
    label?: string;
    disabled?: boolean;
    /** Let the user click the value to type an exact number (can exceed `max`). */
    editable?: boolean;
    onchange?: () => void;
  }
  let {
    value = $bindable(0),
    min = 0,
    max = 100,
    step = 1,
    suffix = "",
    label = "",
    disabled = false,
    editable = false,
    onchange,
  }: Props = $props();

  let editing = $state(false);

  function commit() {
    editing = false;
    if (value < min) value = min;
    onchange?.();
  }
  function focusSelect(node: HTMLInputElement) {
    node.focus();
    node.select();
  }
</script>

<div class="field" class:disabled>
  <div class="row">
    <span class="label">{label}</span>
    {#if editable && editing}
      <input
        class="numedit"
        type="number"
        {min}
        bind:value
        use:focusSelect
        onblur={commit}
        onkeydown={(e) => e.key === "Enter" && commit()}
      />
    {:else if editable}
      <button class="val val-btn" title="Click to type an exact value" onclick={() => (editing = true)}>
        {value}{suffix}
      </button>
    {:else}
      <span class="val">{value}{suffix}</span>
    {/if}
  </div>
  <input type="range" {min} {max} {step} {disabled} bind:value oninput={() => onchange?.()} />
</div>

<style>
  .field {
    width: 100%;
  }
  .field.disabled {
    opacity: 0.4;
  }
  .row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 8px;
  }
  .label {
    color: hsl(var(--muted-foreground));
    font-size: 13px;
  }
  .val {
    font-weight: 700;
    font-size: 15px;
  }
  .val-btn {
    font-family: inherit;
    border-radius: var(--radius-s);
    padding: 1px 6px;
    color: hsl(var(--foreground));
    transition: background 0.12s var(--ease);
  }
  .val-btn:hover {
    background: hsl(var(--glass-border) / 0.12);
  }
  .numedit {
    width: 84px;
    text-align: right;
    height: 28px;
    border-radius: var(--radius-s);
    border: 1px solid hsl(var(--primary) / 0.6);
    background: hsl(var(--glass-bg) / 0.6);
    color: hsl(var(--foreground));
    padding: 0 8px;
    font-size: 14px;
    font-weight: 700;
    font-family: inherit;
  }
  .numedit:focus {
    outline: none;
  }
  input[type="range"] {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 6px;
    border-radius: var(--radius-pill);
    background: hsl(var(--glass-border) / 0.14);
    outline: none;
  }
  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: hsl(var(--primary));
    box-shadow: 0 0 0 4px hsl(var(--primary) / 0.18);
    cursor: pointer;
  }
  input[type="range"]::-moz-range-thumb {
    width: 18px;
    height: 18px;
    border: none;
    border-radius: 50%;
    background: hsl(var(--primary));
    cursor: pointer;
  }
</style>
