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
  background: rgba(14, 18, 28, 0.9);
  border-radius: 18px;
  padding: 12px 14px;
  color: #d6e1f0;
  font-size: 0.85rem;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.05);
  display: grid;
  gap: 10px;
  min-height: 0;
}

.log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  text-transform: uppercase;
  letter-spacing: 0.18em;
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

.log-body::-webkit-scrollbar {
  width: 0;
  height: 0;
}

.log-body {
  scrollbar-width: none;
}
</style>
