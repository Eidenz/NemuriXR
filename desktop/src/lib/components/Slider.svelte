<script lang="ts">
  interface Props {
    value?: number;
    min?: number;
    max?: number;
    step?: number;
    suffix?: string;
    label?: string;
    disabled?: boolean;
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
    onchange,
  }: Props = $props();
</script>

<div class="field" class:disabled>
  <div class="row">
    <span class="label">{label}</span>
    <span class="val">{value}{suffix}</span>
  </div>
  <input
    type="range"
    {min}
    {max}
    {step}
    {disabled}
    bind:value
    oninput={() => onchange?.()}
  />
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
