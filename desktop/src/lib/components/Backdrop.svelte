<!-- Full-bleed cinematic night backdrop, fixed behind everything. Gradient-only
     (no image dependency) — iris glow over deep indigo, with a vignette. -->
<div class="backdrop"></div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: -10;
    overflow: hidden;
    background-color: hsl(var(--background));
    background-image:
      radial-gradient(120% 90% at 80% 6%, hsl(258 72% 26%) 0%, transparent 55%),
      radial-gradient(110% 80% at 8% 102%, hsl(248 60% 17%) 0%, transparent 50%),
      linear-gradient(160deg, hsl(245 28% 11%) 0%, hsl(245 24% 4%) 72%);
  }
  .backdrop::after {
    content: "";
    position: absolute;
    inset: 0;
    box-shadow: inset 0 0 240px 70px hsl(245 40% 2% / 0.85);
  }
</style>
