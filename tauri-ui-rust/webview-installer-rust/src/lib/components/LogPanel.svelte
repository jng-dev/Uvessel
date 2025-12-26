<script lang="ts">
  export let logText = "";
  export let isDone = false;
  export let logBodyEl: HTMLPreElement | null = null;
</script>

<div class="log">
  <div class="log-header">
    <span>Installer log</span>
    {#if isDone}
      <span class="log-status">done</span>
    {/if}
  </div>
  <pre class="log-body" bind:this={logBodyEl}>
{logText || "Preparing installer..."}
  </pre>
</div>

<style>
.log {
  background: rgba(14, 18, 28, 0.84);
  border-radius: 18px;
  padding: 12px 14px;
  color: #d6e1f0;
  font-size: 0.85rem;
  box-shadow:
    inset 0 0 0 1px rgba(255, 255, 255, 0.06),
    0 10px 24px rgba(0, 0, 0, 0.18);
  display: grid;
  gap: 10px;
  min-height: 0;
  position: relative;
  overflow: hidden;
}

.log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  font-size: 0.7rem;
  color: rgba(214, 225, 240, 0.7);
}

.log-status {
  color: #7df6c7;
}

.log-body {
  margin: 0;
  font-family: "JetBrains Mono", "SFMono-Regular", Consolas, monospace;
  white-space: pre-wrap;
  max-height: none;
  min-height: 140px;
  overflow-y: auto;
}

.log::after {
  content: "";
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  height: 1px;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(122, 162, 255, 0.35),
    transparent
  );
  opacity: 0.7;
  pointer-events: none;
}

.log-body::-webkit-scrollbar {
  width: 0;
  height: 0;
}

.log-body {
  scrollbar-width: none;
}
</style>
