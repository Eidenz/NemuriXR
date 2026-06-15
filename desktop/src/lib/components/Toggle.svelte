<script lang="ts">
  interface Props {
    checked?: boolean;
    label?: string;
    onchange?: () => void;
  }
  let { checked = $bindable(false), label = "Toggle", onchange }: Props = $props();
  function toggle() {
    checked = !checked;
    onchange?.();
  }
</script>

<button class="sw" class:on={checked} role="switch" aria-checked={checked} aria-label={label} onclick={toggle}>
  <span class="knob"></span>
</button>

<style>
  .sw {
    width: 52px;
    height: 30px;
    border-radius: var(--radius-pill);
    background: hsl(var(--glass-border) / 0.14);
    transition: background 0.18s var(--ease);
    flex: none;
  }
  .sw.on {
    background: hsl(var(--primary));
  }
  .knob {
    display: block;
    width: 22px;
    height: 22px;
    margin: 4px;
    border-radius: 50%;
    background: #fff;
    transform: translateX(0);
    transition: transform 0.18s var(--ease);
  }
  .sw.on .knob {
    transform: translateX(22px);
  }
</style>
