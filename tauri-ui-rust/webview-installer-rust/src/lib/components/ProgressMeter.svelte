<script lang="ts">
  export let isDone = false;
  export let note = "";
</script>

<div class="meter">
  <div class="track">
    <div class="fill" class:done={isDone}></div>
  </div>
  <p class="note">{note}</p>
</div>

<style>
.meter {
  display: grid;
  gap: 10px;
}

.track {
  height: 9px;
  background: rgba(17, 27, 43, 0.08);
  border-radius: 999px;
  overflow: hidden;
}

.fill {
  height: 100%;
  width: 45%;
  background: linear-gradient(90deg, var(--accent), #8fd3ff, var(--accent));
  animation: glide 2.1s ease-in-out infinite, glow 2.8s ease-in-out infinite;
}

.fill.done {
  width: 100%;
  animation: none;
}

.note {
  margin: 0;
  color: #7b8798;
  font-size: 0.92rem;
}

@keyframes glide {
  0% {
    transform: translateX(-30%);
    width: 35%;
  }
  50% {
    transform: translateX(60%);
    width: 55%;
  }
  100% {
    transform: translateX(-30%);
    width: 35%;
  }
}

@keyframes glow {
  0%,
  100% {
    filter: drop-shadow(0 0 2px rgba(124, 168, 255, 0.35));
  }
  50% {
    filter: drop-shadow(0 0 8px rgba(124, 168, 255, 0.6));
  }
}
</style>
