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
    position: relative;
    width: 52px;
    height: 30px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-pill);
    background: hsl(var(--glass-border) / 0.14);
    transition: background 0.18s var(--ease);
    flex: none;
  }
  .sw.on {
    background: hsl(var(--primary));
  }
  .knob {
    position: absolute;
    top: 50%;
    left: 4px;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: #fff;
    transform: translateY(-50%);
    transition: left 0.18s var(--ease);
  }
  /* 52 − 22 − 4 = 26px from the left edge when on */
  .sw.on .knob {
    left: 26px;
  }
</style>
