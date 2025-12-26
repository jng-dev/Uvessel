<script lang="ts">
  export let withLog = false;
</script>

<section class="card" class:with-log={withLog}>
  <slot />
</section>

<style>
.card {
  width: 100%;
  height: 100%;
  max-height: none;
  background: var(--surface, rgba(255, 255, 255, 0.92));
  border-radius: var(--radius-card, 16px);
  box-shadow: var(--shadow-1), var(--shadow-2);
  border: 1px solid rgba(17, 27, 43, 0.08);
  padding: 22px 24px;
  display: grid;
  gap: 16px;
  overflow: hidden;
  animation: popIn 420ms cubic-bezier(0.2, 0.9, 0.2, 1) both;
  position: relative;
  transform: translateZ(0);
}

.card.with-log {
  grid-template-rows: auto auto minmax(180px, 1fr) auto auto;
}

.card::after {
  content: "";
  position: absolute;
  inset: 1px;
  border-radius: inherit;
  pointer-events: none;
  border: 1px solid rgba(255, 255, 255, 0.65);
  opacity: 0.55;
}

.card::before {
  content: "";
  position: absolute;
  inset: -24px;
  border-radius: 28px;
  pointer-events: none;
  background:
    radial-gradient(closest-side, rgba(122, 162, 255, 0.22), transparent 70%),
    radial-gradient(closest-side, rgba(87, 194, 255, 0.16), transparent 70%);
  filter: blur(18px);
  opacity: 0.55;
  transform: translateY(-10%);
}

@keyframes popIn {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.985);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@media (prefers-reduced-motion: reduce) {
  .card {
    animation: none;
  }
}

@media (max-width: 640px) {
  .card {
    height: calc(100vh - 56px);
    padding: 20px;
  }
}
</style>
