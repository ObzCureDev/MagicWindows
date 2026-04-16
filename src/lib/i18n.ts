import type { Lang } from "./types";

export const translations: Record<string, Record<string, unknown>> = {
  en: {
    appTitle: "MagicWindows",
    appSubtitle: "Apple Magic Keyboard layouts for Windows",
    welcome: {
      title: "Welcome to MagicWindows",
      subtitle: "Fix your Apple keyboard on Windows",
      description:
        "Your Apple Magic Keyboard prints symbols that don't match what Windows types. MagicWindows installs a corrected layout so every keycap matches.",
      detectButton: "Auto-detect my keyboard",
      selectButton: "Choose manually",
    },
    detect: {
      title: "Keyboard Detection",
      instruction: "Find a symbol on your keyboard and press the key it's printed on.",
      charPrompt: "Press the key where you see this symbol",
      charHint: "Find this symbol on your physical keyboard, then press that key. The symbol does not need to appear on screen.",
      noKey: "I don't have this key",
      manual: "Pick manually",
      wrongKey: "Oops, that might be the wrong key. Try again.",
      wrongKeyHelp: "Just press the physical key where you see this symbol. The symbol does not need to appear on screen.",
      failedBanner: "Detection failed. Pick your keyboard manually.",
      back: "Back",
      pressKey: "Press this key:",
      progress: "Key {current} of {total}",
      analyzing: "Analyzing your keyboard...",
      detected: "Detected layout: {name}",
      notDetected: "Could not identify your keyboard layout.",
      tryAgain: "Try again",
      continue: "Continue",
      fallback: "Or choose manually instead",
    },
    select: {
      title: "Select Your Layout",
      instruction: "Choose the Apple keyboard layout that matches your keyboard.",
      searchPlaceholder: "Search layouts...",
      noResults: "No layouts match your search.",
      continue: "Continue",
      back: "Back",
    },
    preview: {
      title: "Layout Preview",
      instruction:
        "Review the layout below. Highlighted keys differ from the default Windows layout.",
      baseLayer: "Base",
      shiftLayer: "Shift",
      altgrLayer: "AltGr",
      altgrShiftLayer: "AltGr+Shift",
      different: "Different from Windows default",
      same: "Same as Windows default",
      installButton: "Install this layout",
      back: "Back",
    },
    install: {
      title: "Installation",
      installing: "Installing layout...",
      success: "Layout installed successfully!",
      error: "Installation failed: {message}",
      adminRequired:
        "Administrator privileges are required. Please restart the app as administrator.",
      openSettings: "Open Windows Settings",
      done: "Done",
    },
    done: {
      title: "All Done!",
      congratulations: "Your Apple Magic Keyboard layout is now installed and added to your input methods.",
      instructions:
        "Press Win+Space to switch to it now. If you don't see it in the list, open Windows Settings to add it manually.",
      switchInfo: "Tip: Win+Space cycles through your installed keyboard layouts.",
      uninstall: "Uninstall layout",
      close: "Close",
    },
    about: {
      title: "About MagicWindows",
      version: "Version {version}",
      description: "Install Apple Magic Keyboard layouts on Windows so every keycap matches what you type.",
      license: "Licensed under Apache 2.0",
      github: "View on GitHub",
      back: "Back",
    },
    common: {
      next: "Next",
      back: "Back",
      cancel: "Cancel",
      loading: "Loading...",
    },
  },
  fr: {
    appTitle: "MagicWindows",
    appSubtitle: "Dispositions Apple Magic Keyboard pour Windows",
    welcome: {
      title: "Bienvenue sur MagicWindows",
      subtitle: "Corrigez votre clavier Apple sous Windows",
      description:
        "Votre Apple Magic Keyboard affiche des symboles qui ne correspondent pas à ce que Windows saisit. MagicWindows installe une disposition corrigée pour que chaque touche corresponde.",
      detectButton: "Détecter mon clavier",
      selectButton: "Choisir manuellement",
    },
    detect: {
      title: "Détection du clavier",
      instruction: "Repérez un symbole sur votre clavier et appuyez sur la touche où il est imprimé.",
      charPrompt: "Appuyez sur la touche où vous voyez ce symbole",
      charHint: "Repérez ce symbole sur votre clavier physique, puis appuyez sur cette touche. Le symbole n'a pas besoin de s'afficher à l'écran.",
      noKey: "Je n'ai pas cette touche",
      manual: "Choisir manuellement",
      wrongKey: "Oups, ce n'est peut-être pas la bonne touche. Réessayez.",
      wrongKeyHelp: "Appuyez simplement sur la touche physique où vous voyez ce symbole. Il n'est pas nécessaire que ce symbole s'affiche à l'écran.",
      failedBanner: "Détection impossible. Choisissez votre clavier manuellement.",
      back: "Retour",
      pressKey: "Appuyez sur cette touche :",
      progress: "Touche {current} sur {total}",
      analyzing: "Analyse de votre clavier…",
      detected: "Disposition détectée : {name}",
      notDetected: "Impossible d'identifier votre disposition clavier.",
      tryAgain: "Réessayer",
      continue: "Continuer",
      fallback: "Ou choisissez manuellement",
    },
    select: {
      title: "Choisir votre disposition",
      instruction:
        "Sélectionnez la disposition Apple qui correspond à votre clavier.",
      searchPlaceholder: "Rechercher…",
      noResults: "Aucune disposition ne correspond à votre recherche.",
      continue: "Continuer",
      back: "Retour",
    },
    preview: {
      title: "Aperçu de la disposition",
      instruction:
        "Vérifiez la disposition ci-dessous. Les touches en surbrillance diffèrent de la disposition Windows par défaut.",
      baseLayer: "Base",
      shiftLayer: "Maj",
      altgrLayer: "AltGr",
      altgrShiftLayer: "AltGr+Maj",
      different: "Différente de Windows par défaut",
      same: "Identique à Windows par défaut",
      installButton: "Installer cette disposition",
      back: "Retour",
    },
    install: {
      title: "Installation",
      installing: "Installation en cours…",
      success: "Disposition installée avec succès !",
      error: "Échec de l'installation : {message}",
      adminRequired:
        "Des privilèges administrateur sont requis. Veuillez relancer l'application en tant qu'administrateur.",
      openSettings: "Ouvrir les paramètres Windows",
      done: "Terminé",
    },
    done: {
      title: "Terminé !",
      congratulations:
        "Votre disposition Apple Magic Keyboard est installée et ajoutée à vos méthodes d'entrée.",
      instructions:
        "Appuyez sur Win+Espace pour l'activer immédiatement. Si elle n'apparaît pas, ouvrez les paramètres Windows pour l'ajouter manuellement.",
      switchInfo:
        "Astuce : Win+Espace fait défiler vos dispositions clavier installées.",
      uninstall: "Désinstaller la disposition",
      close: "Fermer",
    },
    about: {
      title: "À propos de MagicWindows",
      version: "Version {version}",
      description: "Installez les dispositions Apple Magic Keyboard sur Windows pour que chaque touche corresponde à ce que vous tapez.",
      license: "Licence Apache 2.0",
      github: "Voir sur GitHub",
      back: "Retour",
    },
    common: {
      next: "Suivant",
      back: "Retour",
      cancel: "Annuler",
      loading: "Chargement…",
    },
  },
};

/**
 * Retrieve a translated string by dot-separated key path.
 * Supports simple {placeholder} replacement via optional params.
 *
 * Example: t("en", "detect.progress", { current: "2", total: "4" })
 */
export function t(
  lang: Lang,
  key: string,
  params?: Record<string, string>,
): string {
  const parts = key.split(".");
  let value: unknown = translations[lang];
  for (const part of parts) {
    if (value && typeof value === "object") {
      value = (value as Record<string, unknown>)[part];
    } else {
      return key; // fallback: return the key itself
    }
  }
  if (typeof value !== "string") return key;

  if (params) {
    let result = value;
    for (const [k, v] of Object.entries(params)) {
      result = result.replace(`{${k}}`, v);
    }
    return result;
  }
  return value;
}
