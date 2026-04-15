<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores";
  import { t } from "../lib/i18n";

  async function openSettings() {
    try {
      const { open } = await import("@tauri-apps/plugin-shell");
      await open("ms-settings:regionlanguage");
    } catch (e) {
      console.error("Could not open settings:", e);
    }
  }

  async function uninstall() {
    if (!appState.selectedLayoutId) return;
    try {
      await invoke("uninstall_layout", { id: appState.selectedLayoutId });
      appState.selectedLayoutId = null;
      appState.page = "welcome";
    } catch (err) {
      console.error("Uninstall failed:", err);
      appState.error = String(err);
    }
  }

  function close() {
    try {
      // Tauri v2 window close
      import("@tauri-apps/api/window").then(({ getCurrentWindow }) => {
        getCurrentWindow().close();
      });
    } catch {
      window.close();
    }
  }
</script>

<div class="page">
  <div class="page__content">
    <div class="checkmark">&#10003;</div>

    <div class="page__header">
      <h1 class="page__title">{t(appState.lang, "done.title")}</h1>
      <p class="page__subtitle">{t(appState.lang, "done.congratulations")}</p>
    </div>

    <div class="status status--info" style="max-width: 460px;">
      {t(appState.lang, "done.switchInfo")}
    </div>

    <p class="text-secondary text-center" style="max-width: 460px;">
      {t(appState.lang, "done.instructions")}
    </p>

    <div class="page__actions">
      <button class="btn btn-secondary" onclick={openSettings}>
        {t(appState.lang, "install.openSettings")}
      </button>
      <button class="btn btn-primary" onclick={close}>
        {t(appState.lang, "done.close")}
      </button>
    </div>

    <div class="mt-4">
      <button class="btn btn-danger" onclick={uninstall}>
        {t(appState.lang, "done.uninstall")}
      </button>
    </div>
  </div>
</div>
