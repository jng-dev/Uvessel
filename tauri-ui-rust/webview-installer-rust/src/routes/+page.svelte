<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Image } from "@tauri-apps/api/image";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import InstallerHeader from "$lib/components/InstallerHeader.svelte";
  import InstallerShell from "$lib/components/InstallerShell.svelte";
  import InstallerCard from "$lib/components/InstallerCard.svelte";
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
              try {
                await invoke("focus_window");
              } catch {
                // Ignore focus errors.
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
              try {
                await invoke("focus_window");
              } catch {
                // Ignore focus errors.
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
      await invoke("focus_window");
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

<InstallerShell updating={isUpdate}>
  <InstallerCard withLog={logEnabled}>
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
  </InstallerCard>
</InstallerShell>
