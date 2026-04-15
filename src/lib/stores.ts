import type { Page, Lang, LayoutMeta } from "./types";

class AppState {
  page = $state<Page>("welcome");
  lang = $state<Lang>("fr");
  selectedLayoutId = $state<string | null>(null);
  layouts = $state<LayoutMeta[]>([]);
  error = $state<string | null>(null);
  installing = $state(false);
}

export const appState = new AppState();
