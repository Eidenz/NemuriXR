<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title?: string;
    desc?: string;
    children?: Snippet;
  }
  let { title, desc, children }: Props = $props();
</script>

<section class="glass card">
  {#if title}
    <header class="head">
      <h3>{title}</h3>
      {#if desc}<p>{desc}</p>{/if}
    </header>
  {/if}
  {@render children?.()}
</section>

<style>
  .card {
    border-radius: var(--radius-l);
    padding: 22px;
  }
  .head {
    margin-bottom: 16px;
  }
  .head h3 {
    font-size: 17px;
  }
  .head p {
    margin: 6px 0 0;
    color: hsl(var(--muted-foreground));
    font-size: 13.5px;
    line-height: 1.45;
  }
</style>
