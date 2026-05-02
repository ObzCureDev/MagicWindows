// Bilingual EN / FR strings for the SPA. The toggle in the footer flips
// between the two; the choice is persisted in localStorage and seeded from
// navigator.language on first load.

export type Lang = "en" | "fr";

const STORAGE_KEY = "mw-lang";

function detectInitial(): Lang {
  if (typeof window === "undefined") return "en";
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "en" || stored === "fr") return stored;
  const nav = (navigator.language ?? "en").toLowerCase();
  return nav.startsWith("fr") ? "fr" : "en";
}

let lang = $state<Lang>(detectInitial());

export const i18n = {
  get lang(): Lang {
    return lang;
  },
  set lang(value: Lang) {
    lang = value;
    if (typeof window !== "undefined") {
      localStorage.setItem(STORAGE_KEY, value);
      document.documentElement.lang = value;
    }
  },
  toggle() {
    this.lang = lang === "en" ? "fr" : "en";
  },
};

const STRINGS = {
  // Home
  "home.h1": {
    en: "Apple keyboard layouts on Windows",
    fr: "Layouts Apple sur Windows",
  },
  "home.lede1": {
    en: "Plug your Apple Magic Keyboard into a Windows PC and the printed keys don't match what you type. @, #, accented letters, even Shift+Enter — Windows ignores half the keycap labels.",
    fr: "Branchez un Magic Keyboard Apple sur un PC Windows et les touches imprimées ne correspondent pas à ce que vous tapez. @, #, lettres accentuées, même Shift+Entrée — Windows ignore la moitié des inscriptions.",
  },
  "home.lede2": {
    en: "Download the matching layout for your keyboard, double-click the installer, restart. Done.",
    fr: "Téléchargez le layout adapté à votre clavier, double-cliquez sur l'installeur, redémarrez. C'est tout.",
  },
  "home.secondaryCta": {
    en: "Want auto-detect & advanced options? →",
    fr: "Auto-détection et options avancées ? →",
  },
  "home.cardCta": {
    en: "Preview & download →",
    fr: "Aperçu & téléchargement →",
  },

  // Preview
  "preview.back": { en: "← All layouts", fr: "← Tous les layouts" },
  "preview.switcherLabel": { en: "Choose a layout", fr: "Choisir un layout" },
  "preview.layerLabel": { en: "Modifier layer", fr: "Couche modificateur" },
  "preview.layer.base": { en: "Base", fr: "Base" },
  "preview.layer.shift": { en: "Shift", fr: "Shift" },
  "preview.layer.altgr": { en: "AltGr", fr: "AltGr" },
  "preview.layer.altgrShift": { en: "AltGr+Shift", fr: "AltGr+Shift" },
  "preview.dlPrompt": {
    en: "Pick the build that matches your PC:",
    fr: "Choisissez le build correspondant à votre PC :",
  },
  "preview.dlUnavailableArch": { en: "unavailable", fr: "indisponible" },
  "preview.dlHelpX64": {
    en: "for most PCs (Intel / AMD).",
    fr: "pour la plupart des PC (Intel / AMD).",
  },
  "preview.dlHelpArm64": {
    en: "for Snapdragon / Surface Pro X / Copilot+ PCs.",
    fr: "pour les PC Snapdragon / Surface Pro X / Copilot+.",
  },
  "preview.dlUnavailable": {
    en: "Downloads temporarily unavailable. Please try again later.",
    fr: "Téléchargements temporairement indisponibles. Réessayez plus tard.",
  },

  // Desktop
  "desktop.back": { en: "← All layouts", fr: "← Tous les layouts" },
  "desktop.h1": { en: "The desktop app", fr: "L'application desktop" },
  "desktop.lede": {
    en: "The web ZIP installs only the layout. The desktop app does the same install, plus a handful of features that need a native client.",
    fr: "Le ZIP web installe uniquement le layout. L'application desktop fait la même installation, plus quelques fonctionnalités qui nécessitent un client natif.",
  },
  "desktop.col.web": { en: "Web ZIP", fr: "ZIP web" },
  "desktop.col.desktop": { en: "Desktop app", fr: "App desktop" },
  "desktop.row.installLayout": {
    en: "Install a layout",
    fr: "Installer un layout",
  },
  "desktop.row.autoDetect": {
    en: "Auto-detect your keyboard",
    fr: "Auto-détection du clavier",
  },
  "desktop.row.modifiers": {
    en: "Mac-style modifier toggles (Cmd↔Ctrl, Caps→Ctrl)",
    fr: "Modificateurs façon Mac (Cmd↔Ctrl, Caps→Ctrl)",
  },
  "desktop.row.f12": {
    en: "F12 / Eject remap (Calculator, Search, Mute…)",
    fr: "Remap F12 / Eject (Calculatrice, Recherche, Muet…)",
  },
  "desktop.row.healthCheck": {
    en: "Health check after install",
    fr: "Vérification post-installation",
  },
  "desktop.row.uninstall": {
    en: "One-click uninstall UI",
    fr: "Désinstallation en un clic",
  },
  "desktop.cta": {
    en: "Download the desktop app",
    fr: "Télécharger l'application desktop",
  },
  "desktop.note": {
    en: "Latest release on GitHub. Windows 10/11, x64.",
    fr: "Dernière release sur GitHub. Windows 10/11, x64.",
  },

  // Footer
  "footer.madeBy": { en: "Made by", fr: "Réalisé par" },
  "footer.github": { en: "GitHub", fr: "GitHub" },
  "footer.bug": { en: "Report a bug", fr: "Signaler un bug" },
  "footer.disclaimer": {
    en: "MagicWindows is open source (MIT). Apple, Magic Keyboard, and macOS are trademarks of Apple Inc. Windows is a trademark of Microsoft Corporation. This project is not affiliated with either.",
    fr: "MagicWindows est open source (MIT). Apple, Magic Keyboard et macOS sont des marques d'Apple Inc. Windows est une marque de Microsoft Corporation. Ce projet n'est affilié à aucune des deux.",
  },
  "footer.langToggleAriaLabel": {
    en: "Switch language",
    fr: "Changer la langue",
  },
} as const satisfies Record<string, Record<Lang, string>>;

export type StringKey = keyof typeof STRINGS;

/** Translate a key into the active language. Reactive — calling this inside
 *  a Svelte template re-runs when `i18n.lang` changes. */
export function t(key: StringKey): string {
  return STRINGS[key][i18n.lang] ?? STRINGS[key].en;
}
