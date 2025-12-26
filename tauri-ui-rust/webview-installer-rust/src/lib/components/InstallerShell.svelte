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
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.shell {
  width: 100%;
  height: 100%;
  min-height: 100vh;
  padding: 10px;
  --bg0: #f4f7fa;
  --bg1: #eef2f6;
  --bg2: #e6edf4;
  --surface: rgba(255, 255, 255, 0.92);
  --surface-solid: #ffffff;
  --text: #121117;
  --muted: rgba(18, 17, 23, 0.62);
  --radius-window: 12px;
  --radius-card: 16px;
  --shadow-1: 0 8px 20px rgba(18, 24, 40, 0.1);
  --shadow-2: 0 22px 50px rgba(18, 24, 40, 0.14);
  --accent: #7aa2ff;
  --accent2: #57c2ff;
  background:
    radial-gradient(circle at top left, rgba(124, 168, 255, 0.18), transparent 55%),
    radial-gradient(circle at 20% 80%, rgba(152, 214, 255, 0.18), transparent 60%),
    linear-gradient(155deg, var(--bg0) 0%, var(--bg1) 60%, var(--bg2) 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  /* padding: 42px 14px 14px; */
  position: relative;
  overflow: hidden;
  border-radius: var(--radius-window);
  --accent-soft: rgba(104, 140, 255, 0.18);
  border: none;
}

.shell::before {
  content: "";
  position: absolute;
  inset: 0;
  pointer-events: none;
  background: linear-gradient(
    115deg,
    rgba(122, 162, 255, 0.1),
    rgba(255, 255, 255, 0.0) 35%,
    rgba(87, 194, 255, 0.08) 70%,
    rgba(255, 255, 255, 0.0)
  );
  opacity: 0.55;
  transform: translateX(-20%);
  animation: sheen 9s ease-in-out infinite;
}

.shell.updating {
  --accent: #57c2ff;
  --accent-soft: rgba(87, 194, 255, 0.18);
}

@keyframes sheen {
  0%,
  100% {
    transform: translateX(-22%);
  }
  50% {
    transform: translateX(10%);
  }
}

@media (prefers-reduced-motion: reduce) {
  .shell::before {
    animation: none;
  }
}

@media (max-width: 640px) {
  .shell {
    padding: 38px 12px 12px;
  }
}
</style>
