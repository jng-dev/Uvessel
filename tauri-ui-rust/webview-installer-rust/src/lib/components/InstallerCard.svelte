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
  background: #ffffff;
  border-radius: var(--radius-card, 16px);
  padding: 22px 24px;
  display: grid;
  gap: 16px;
  overflow: hidden;
  animation: fadeUp 0.6s ease-out;
  position: relative;
  border: none;
}

.card.with-log {
  grid-template-rows: auto auto minmax(180px, 1fr) auto auto;
}

@keyframes fadeUp {
  from {
    opacity: 0;
    transform: translateY(18px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 640px) {
  .card {
    height: calc(100vh - 56px);
    padding: 20px;
  }
}
</style>
