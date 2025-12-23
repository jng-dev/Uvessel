<script lang="ts">
  export let statusText = "";
  export let isDone = false;
  export let isFailed = false;
</script>

<div class="footer">
  <span class="pulse" class:done={isDone} class:failed={isFailed}></span>
  <span>{statusText}</span>
</div>

<style>
.footer {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 0.95rem;
  color: #2d3645;
}

.pulse {
  width: 10px;
  height: 10px;
  border-radius: 999px;
  background: var(--accent);
  box-shadow: 0 0 0 6px var(--accent-soft);
  animation: pulse 1.6s ease-in-out infinite;
}

.pulse.done {
  background: #2db67d;
  box-shadow: 0 0 0 6px rgba(45, 182, 125, 0.18);
  animation: none;
}

.pulse.failed {
  background: #ff6b6b;
  box-shadow: 0 0 0 6px rgba(255, 107, 107, 0.2);
  animation: none;
}

@keyframes pulse {
  0%,
  100% {
    transform: scale(1);
    opacity: 0.8;
  }
  50% {
    transform: scale(1.2);
    opacity: 1;
  }
}
</style>
