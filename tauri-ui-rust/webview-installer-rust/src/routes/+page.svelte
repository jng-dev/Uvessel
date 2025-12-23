<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Image } from "@tauri-apps/api/image";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  type InstallUiInfo = {
    name: string;
    icon_path?: string | null;
    done_file?: string | null;
  };

  let appName = "Your App";
  let iconUrl = "";
  let initial = "A";
  let isDone = false;
  let pollTimer: number | undefined;

  async function loadIcon(path: string) {
    try {
      const image = await Image.fromPath(path);
      const size = await image.size();
      const rgba = await image.rgba();
      const canvas = document.createElement("canvas");
      canvas.width = size.width;
      canvas.height = size.height;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      const data = new ImageData(
        new Uint8ClampedArray(rgba),
        size.width,
        size.height
      );
      ctx.putImageData(data, 0, 0);
      iconUrl = canvas.toDataURL("image/png");
    } catch {
      // Ignore icon load errors.
    }
  }

  onMount(async () => {
    try {
      const info = await invoke<InstallUiInfo>("get_install_ui_info");
      if (info?.name) {
        appName = info.name;
        initial = info.name.trim().charAt(0).toUpperCase() || "A";
      }
      if (info?.icon_path) {
        await loadIcon(info.icon_path);
      }
      if (info?.done_file) {
        pollTimer = window.setInterval(async () => {
          try {
            const done = await invoke<boolean>("is_install_done");
            if (done) {
              isDone = true;
              if (pollTimer) {
                clearInterval(pollTimer);
              }
            }
          } catch {
            // Ignore polling errors.
          }
        }, 800);
      }
    } catch {
      initial = appName.trim().charAt(0).toUpperCase() || "A";
    }
    try {
      await getCurrentWindow().center();
    } catch {
      // Ignore if permission denied.
    }
  });

  onDestroy(() => {
    if (pollTimer) {
      clearInterval(pollTimer);
    }
  });

  async function closeWindow() {
    try {
      await invoke("close_window");
    } catch {
      // Ignore close errors.
    }
  }

</script>

<main class="shell">
  <div class="titlebar" data-tauri-drag-region>
    <span class="title" data-tauri-drag-region>Installing</span>
  </div>

  <section class="card">
    <div class="header">
      <div class="icon-wrap">
        {#if iconUrl}
          <img class="icon" src={iconUrl} alt="App icon" />
        {:else}
          <div class="icon-fallback">{initial}</div>
        {/if}
      </div>
      <div class="title-block">
        <p class="eyebrow">Installing</p>
        <h1>{appName}</h1>
        <p class="subtitle">
          {isDone
            ? "Install complete. Click launch to continue."
            : "Setting things up for the first run."}
        </p>
      </div>
    </div>

    <div class="meter">
      <div class="track">
        <div class="fill"></div>
      </div>
      <p class="note">
        {isDone
          ? "All set. You're ready to launch."
          : "This can take a minute. We'll let you know when it's ready."}
      </p>
    </div>

    <div class="footer">
      <span class="pulse" class:done={isDone}></span>
      <span>{isDone ? "Ready to launch" : "Preparing runtime environment"}</span>
    </div>

    {#if isDone}
      <button class="primary" on:click={closeWindow}>
        Launch {appName}
      </button>
    {/if}
  </section>
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
}

:global(*),
:global(*::before),
:global(*::after) {
  box-sizing: inherit;
}

:global(body) {
  overscroll-behavior: none;
}

.shell {
  min-height: 100vh;
  background: linear-gradient(155deg, #f4f7fa 0%, #eef2f6 60%, #e6edf4 100%);
  display: grid;
  place-items: center;
  padding: 24px;
  position: relative;
  overflow: hidden;
}

.titlebar {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 16px;
  color: #5f6a79;
  font-size: 0.78rem;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  background: rgba(245, 248, 252, 0.8);
  backdrop-filter: blur(8px);
  border-bottom: 1px solid rgba(17, 27, 43, 0.06);
  z-index: 2;
}

.card {
  width: min(640px, calc(100vw - 48px));
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid rgba(17, 27, 43, 0.08);
  border-radius: 26px;
  padding: 30px;
  box-shadow:
    0 22px 50px rgba(18, 24, 40, 0.16),
    0 0 0 1px rgba(104, 140, 255, 0.08),
    0 0 24px rgba(104, 140, 255, 0.16);
  display: grid;
  gap: 22px;
  animation: fadeUp 0.6s ease-out;
  backdrop-filter: blur(8px);
}

.header {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 20px;
  align-items: center;
}

.icon-wrap {
  width: 72px;
  height: 72px;
  border-radius: 22px;
  background: linear-gradient(140deg, #101722, #1b2433);
  display: grid;
  place-items: center;
  box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.1);
}

.icon {
  width: 52px;
  height: 52px;
  object-fit: contain;
}

.icon-fallback {
  width: 52px;
  height: 52px;
  border-radius: 16px;
  background: linear-gradient(140deg, #ccd8e6, #b5c1d6);
  color: #1b2330;
  display: grid;
  place-items: center;
  font-weight: 600;
  font-size: 1.4rem;
}

.eyebrow {
  margin: 0;
  font-size: 0.95rem;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: #7b8798;
}

h1 {
  margin: 6px 0 6px;
  font-size: clamp(2rem, 4vw, 2.8rem);
  color: #18202c;
}

.subtitle {
  margin: 0;
  color: #6f7a8b;
  font-size: 1.05rem;
}

.meter {
  display: grid;
  gap: 12px;
}

.track {
  height: 10px;
  background: rgba(17, 27, 43, 0.08);
  border-radius: 999px;
  overflow: hidden;
}

.fill {
  height: 100%;
  width: 45%;
  background: linear-gradient(90deg, #7aa2ff, #8fd3ff, #7aa2ff);
  animation: glide 2.2s ease-in-out infinite;
}

.note {
  margin: 0;
  color: #7b8798;
  font-size: 0.95rem;
}

.footer {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 0.95rem;
  color: #2d3645;
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
  align-self: center;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  box-shadow: 0 14px 30px rgba(23, 34, 54, 0.28);
}

.primary:hover {
  transform: translateY(-1px);
}

.pulse {
  width: 10px;
  height: 10px;
  border-radius: 999px;
  background: #6a8cff;
  box-shadow: 0 0 0 6px rgba(106, 140, 255, 0.18);
  animation: pulse 1.6s ease-in-out infinite;
}

.pulse.done {
  background: #2db67d;
  box-shadow: 0 0 0 6px rgba(45, 182, 125, 0.18);
  animation: none;
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
    padding: 24px;
  }

  .header {
    grid-template-columns: 1fr;
    text-align: center;
  }

  .icon-wrap {
    margin: 0 auto;
  }

  .footer {
    justify-content: center;
  }
}
</style>


