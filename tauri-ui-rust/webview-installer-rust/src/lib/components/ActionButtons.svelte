<script lang="ts">
  export let appName = "Your App";
  export let showLaunch = false;
  export let showClose = false;
  export let onLaunch: () => void = () => {};
  export let onClose: () => void = () => {};
</script>

{#if showLaunch || showClose}
  <div class="actions">
    {#if showLaunch}
      <button class="primary" on:click={onLaunch}>
        Launch {appName}
      </button>
    {/if}
    {#if showClose}
      <button class="ghost" on:click={onClose}>
        Close
      </button>
    {/if}
  </div>
{/if}

<style>
.actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.primary {
  border: none;
  padding: 12px 18px;
  border-radius: 999px;
  background: linear-gradient(120deg, #101722, #233149);
  color: #fff;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  box-shadow: 0 14px 30px rgba(23, 34, 54, 0.28);
}

.primary:hover {
  transform: translateY(-1px) scale(1.01);
  box-shadow:
    0 18px 34px rgba(23, 34, 54, 0.32),
    0 0 14px rgba(124, 168, 255, 0.28);
}

.ghost {
  border: 1px solid rgba(16, 23, 34, 0.18);
  background: rgba(255, 255, 255, 0.6);
  color: #1b2330;
  padding: 12px 18px;
  border-radius: 999px;
  font-size: 1rem;
  cursor: pointer;
  transition: transform 0.2s ease, box-shadow 0.2s ease, border-color 0.2s ease;
}

.ghost:hover {
  transform: translateY(-1px);
  border-color: rgba(124, 168, 255, 0.6);
  box-shadow:
    0 10px 22px rgba(23, 34, 54, 0.18),
    0 0 16px rgba(124, 168, 255, 0.24);
}
</style>
