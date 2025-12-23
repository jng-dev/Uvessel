<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Image } from "@tauri-apps/api/image";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import InstallerHeader from "$lib/components/InstallerHeader.svelte";
  import ProgressMeter from "$lib/components/ProgressMeter.svelte";
  import LogPanel from "$lib/components/LogPanel.svelte";
  import StatusFooter from "$lib/components/StatusFooter.svelte";
  import ActionButtons from "$lib/components/ActionButtons.svelte";

  type InstallUiInfo = {
    name: string;
    icon_path?: string | null;
    done_file?: string | null;
    version?: string | null;
    mode?: string | null;
    log_file?: string | null;
  };

  let appName = "Your App";
  let iconUrl = "";
  let initial = "A";
  let isDone = false;
  let isUpdate = false;
  let isFailed = false;
  let versionLabel = "";
  let logText = "";
  let logOffset = 0;
  let logEnabled = false;
  let logBodyEl: HTMLPreElement | null = null;
  let pollTimer: number | undefined;
  let logTimer: number | undefined;
  let didAutoClose = false;

  $: eyebrow = isUpdate ? "Updating" : "Installing";
  $: subtitle = isFailed
    ? "Installation failed. Please check the log."
    : isDone
      ? isUpdate
        ? "Update complete. Restarting shortly."
        : "Install complete. Click launch to continue."
      : isUpdate
        ? "Applying the latest release."
        : "Setting things up for the first run.";
  $: note = isFailed
    ? "Something went wrong. You can close and retry."
    : isDone
      ? isUpdate
        ? "Update applied. Finishing up."
        : "All set. You're ready to launch."
      : "This can take a minute. We'll let you know when it's ready.";
  $: footerText = isFailed
    ? "Install failed"
    : isDone
      ? isUpdate
        ? "Update complete"
        : "Ready to launch"
      : "Preparing runtime environment";
  $: showLaunch = isDone && !isUpdate;
  $: showClose = isFailed || showLaunch;

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
      if (info?.version) {
        versionLabel = `v${info.version}`;
      }
      if (info?.mode && info.mode.toLowerCase() === "update") {
        isUpdate = true;
      }
      if (info?.log_file) {
        logEnabled = true;
      }
      if (info?.icon_path) {
        await loadIcon(info.icon_path);
      }
      if (info?.done_file) {
        pollTimer = window.setInterval(async () => {
          try {
            const status = await invoke<{ status: string }>(
              "get_install_status"
            );
            if (status?.status === "ok") {
              isDone = true;
              isFailed = false;
              if (pollTimer) {
                clearInterval(pollTimer);
              }
              if (isUpdate && !didAutoClose) {
                didAutoClose = true;
                window.setTimeout(closeWindow, 600);
              }
            } else if (status?.status === "fail") {
              isDone = false;
              isFailed = true;
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
    if (logEnabled) {
      logTimer = window.setInterval(async () => {
        try {
          const chunk = await invoke<{ text: string; next_offset: number }>(
            "read_install_log",
            { offset: logOffset, maxBytes: 8192 }
          );
          if (chunk?.text) {
            logOffset = chunk.next_offset;
            logText = `${logText}${chunk.text}`;
            const lines = logText.split("\n");
            if (lines.length > 200) {
              logText = lines.slice(-200).join("\n");
            }
            await tick();
            if (logBodyEl) {
              logBodyEl.scrollTop = logBodyEl.scrollHeight;
            }
          }
        } catch {
          // Ignore log polling errors.
        }
      }, 250);
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
    if (logTimer) {
      clearInterval(logTimer);
    }
  });

  async function closeWindow() {
    try {
      await invoke("close_window");
    } catch {
      // Ignore close errors.
    }
  }

  async function launchAndClose() {
    try {
      await invoke("mark_launch_requested");
    } catch {
      // Ignore mark errors.
    }
    await closeWindow();
  }
</script>

<main class="shell" class:updating={isUpdate}>
  <div class="titlebar" data-tauri-drag-region>
    <span class="title" data-tauri-drag-region>{eyebrow}</span>
  </div>

  <section class="card" class:with-log={logEnabled}>
    <InstallerHeader
      {appName}
      {versionLabel}
      {eyebrow}
      {subtitle}
      {iconUrl}
      {initial}
    />

    <ProgressMeter isDone={isDone} {note} />

    {#if logEnabled}
      <LogPanel bind:logBodyEl {logText} isDone={isDone || isFailed} />
    {/if}

    <StatusFooter statusText={footerText} {isDone} {isFailed} />

    <ActionButtons
      {appName}
      showLaunch={showLaunch}
      showClose={showClose}
      onLaunch={launchAndClose}
      onClose={closeWindow}
    />
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
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 42px 14px 14px;
  position: relative;
  overflow: hidden;
  --accent: #7aa2ff;
  --accent-soft: rgba(104, 140, 255, 0.18);
}

.shell.updating {
  --accent: #57c2ff;
  --accent-soft: rgba(87, 194, 255, 0.18);
}

.titlebar {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 16px;
  color: #5f6a79;
  font-size: 0.74rem;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  background: rgba(245, 248, 252, 0.8);
  backdrop-filter: blur(8px);
  border-bottom: 1px solid rgba(17, 27, 43, 0.06);
  z-index: 2;
}

.card {
  width: min(640px, 100%);
  height: calc(100vh - 64px);
  max-height: 520px;
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid rgba(17, 27, 43, 0.08);
  border-radius: 24px;
  padding: 22px 24px;
  box-shadow:
    0 22px 50px rgba(18, 24, 40, 0.16),
    0 0 0 1px rgba(104, 140, 255, 0.08),
    0 0 24px rgba(104, 140, 255, 0.16);
  display: grid;
  gap: 16px;
  overflow: hidden;
  animation: fadeUp 0.6s ease-out;
  backdrop-filter: blur(8px);
}

.card.with-log {
  grid-template-rows: auto auto minmax(180px, 1fr) auto auto;
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
  .shell {
    padding: 38px 12px 12px;
  }

  .card {
    height: calc(100vh - 56px);
    padding: 20px;
  }
}
</style>
