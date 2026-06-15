<script lang="ts">
  interface Props {
    title: string;
    desc: string;
  }
  let { title, desc }: Props = $props();
</script>

<div class="ph">
  <svg viewBox="0 0 24 24" width="48" height="48" aria-hidden="true">
    <path fill="currentColor" d="M12.7 2a8 8 0 1 0 9.3 9.3 7 7 0 0 1-9.3-9.3Z" />
  </svg>
  <h2>{title}</h2>
  <p>{desc}</p>
</div>

<style>
  .ph {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    height: 100%;
    gap: 10px;
    color: hsl(var(--muted-foreground));
  }
  .ph svg {
    color: hsl(var(--primary) / 0.5);
  }
  .ph h2 {
    color: hsl(var(--foreground));
    font-size: 22px;
  }
  .ph p {
    margin: 0;
    max-width: 440px;
    line-height: 1.5;
  }
</style>
