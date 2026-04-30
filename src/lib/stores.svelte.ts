import type { Page, Lang, Theme, LayoutMeta, HealthCheckTarget } from "./types";

function detectLang(): Lang {
  const nav = globalThis.navigator;
  if (!nav) return "en";
  const primary = nav.language ?? nav.languages?.[0] ?? "en";
  return primary.startsWith("fr") ? "fr" : "en";
}

class AppState {
  page = $state<Page>("welcome");
  lang = $state<Lang>(detectLang());
  selectedLayoutId = $state<string | null>(null);
  layouts = $state<LayoutMeta[]>([]);
  theme = $state<Theme>("system");
  error = $state<string | null>(null);
  installing = $state(false);
  lastInstallMs = $state<number | null>(null);
  detectionFailedMessage = $state<string | null>(null);
  healthCheckTarget = $state<HealthCheckTarget | null>(null);
}

export const appState = new AppState();
