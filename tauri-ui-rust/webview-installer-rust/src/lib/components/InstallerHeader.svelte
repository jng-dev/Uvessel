<script lang="ts">
  export let appName = "Your App";
  export let versionLabel = "";
  export let eyebrow = "Installing";
  export let subtitle = "";
  export let iconUrl = "";
  export let initial = "A";
</script>

<div class="header">
  <div class="icon-wrap">
    {#if iconUrl}
      <img class="icon" src={iconUrl} alt="App icon" />
    {:else}
      <div class="icon-fallback">{initial}</div>
    {/if}
  </div>
  <div class="title-block">
    <p class="eyebrow">{eyebrow}</p>
    <h1>{appName}</h1>
    {#if versionLabel}
      <p class="version">{versionLabel}</p>
    {/if}
    <p class="subtitle">{subtitle}</p>
  </div>
</div>

<style>
.header {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 18px;
  align-items: center;
}

.icon-wrap {
  width: 68px;
  height: 68px;
  border-radius: 20px;
  background: linear-gradient(140deg, #101722, #1b2433);
  display: grid;
  place-items: center;
  box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.1);
}

.icon {
  width: 48px;
  height: 48px;
  object-fit: contain;
}

.icon-fallback {
  width: 48px;
  height: 48px;
  border-radius: 14px;
  background: linear-gradient(140deg, #ccd8e6, #b5c1d6);
  color: #1b2330;
  display: grid;
  place-items: center;
  font-weight: 600;
  font-size: 1.25rem;
}

.eyebrow {
  margin: 0;
  font-size: 0.88rem;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: #7b8798;
}

h1 {
  margin: 4px 0 4px;
  font-size: clamp(1.9rem, 3.6vw, 2.6rem);
  color: #18202c;
}

.version {
  margin: 0 0 6px;
  font-size: 0.9rem;
  color: #8b95a6;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.subtitle {
  margin: 0;
  color: #6f7a8b;
  font-size: 1.02rem;
}

@media (max-width: 640px) {
  .header {
    grid-template-columns: 1fr;
    text-align: center;
  }

  .icon-wrap {
    margin: 0 auto;
  }
}
</style>
