import type { Page, Lang, Theme, LayoutMeta } from "./types";

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
}

export const appState = new AppState();
