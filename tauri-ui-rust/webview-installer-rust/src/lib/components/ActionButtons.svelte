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
  transform: translateY(-1px);
}

.ghost {
  border: 1px solid rgba(16, 23, 34, 0.18);
  background: transparent;
  color: #1b2330;
  padding: 12px 18px;
  border-radius: 999px;
  font-size: 1rem;
  cursor: pointer;
}
</style>
