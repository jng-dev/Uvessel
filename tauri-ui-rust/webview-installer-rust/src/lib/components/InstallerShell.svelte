<script lang="ts">
  export let updating = false;
</script>

<main class="shell" class:updating={updating}>
  <slot />
</main>

<style>
@import url("https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600&display=swap");

:global(html),
:global(body) {
  margin: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  font-family: "Space Grotesk", "Segoe UI", sans-serif;
  color: #121117;
  box-sizing: border-box;
  background: transparent;
  border-radius: 0;
  border: none;
}

:global(*),
:global(*::before),
:global(*::after) {
  box-sizing: inherit;
}

:global(body) {
  overscroll-behavior: none;
  user-select: none;
  -webkit-user-select: none;
  -ms-user-select: none;
  line-height: 1.35;
}

.shell {
  width: 100%;
  height: 100%;
  min-height: 100vh;
  padding: 10px;
  --radius-window: 12px;
  --radius-card: 16px;
  background:
    radial-gradient(circle at top left, rgba(124, 168, 255, 0.18), transparent 55%),
    radial-gradient(circle at 20% 80%, rgba(152, 214, 255, 0.18), transparent 60%),
    linear-gradient(155deg, #f4f7fa 0%, #eef2f6 60%, #e6edf4 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  /* padding: 42px 14px 14px; */
  position: relative;
  overflow: hidden;
  border-radius: var(--radius-window);
  --accent: #7aa2ff;
  --accent-soft: rgba(104, 140, 255, 0.18);
  border: none;
}

.shell.updating {
  --accent: #57c2ff;
  --accent-soft: rgba(87, 194, 255, 0.18);
}

@media (max-width: 640px) {
  .shell {
    padding: 38px 12px 12px;
  }
}
</style>
