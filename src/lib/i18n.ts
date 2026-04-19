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
      elapsed: "Installed in {seconds}s",
      error: "Installation failed: {message}",
      adminRequired:
        "Administrator privileges are required. Please restart the app as administrator.",
      openSettings: "Open Windows Settings",
      done: "Done",
    },
    test: {
      title: "Test your keyboard",
      hint: "Type below and check that every key prints what's on its keycap.",
      placeholder: "Try typing here…",
      continue: "Looks good",
    },
    done: {
      title: "All Done!",
      congratulations: "Your Apple Magic Keyboard layout is now installed and added to your input methods.",
      instructions:
        "Press Win+Space to switch to it now. If you don't see it in the list, open Windows Settings to add it manually.",
      switchInfo: "Tip: Win+Space cycles through your installed keyboard layouts.",
      uninstall: "Uninstall layout",
      close: "Close",
      modOfferTitle: "Mac-style shortcuts on Windows?",
      modOfferHint: "Optional — applies system-wide and requires a reboot.",
      modNone: "No thanks, keep as-is",
      modMac: "macOS shortcuts (Cmd+C, Cmd+V)",
      modMacDesc: "Swaps Cmd ↔ Ctrl so your Apple muscle memory works.",
      modWin: "Windows-strict layout (Ctrl · Win · Alt)",
      modWinDesc: "Swaps Option ↔ Cmd so the physical key order matches a PC.",
      modApply: "Apply (reboot required)",
      modApplying: "Writing to registry…",
      modApplied: "Done. Reboot when convenient.",
      removeWindowsDefault:
        "Windows still has its default {locale} layout installed. Removing it prevents confusion when switching via Win+Space.",
      removeWindowsDefaultBtn: "Remove the Windows default {locale} layout",
      windowsDefaultRemoved: "Windows layout removed ✓",
    },
    settings: {
      topbarTitle: "Settings",
      title: "Installed keyboard layouts",
      subtitle:
        "Review and remove any keyboard layout on your system.",
      badgeMagic: "MagicWindows",
      badgeSystem: "System",
      badgeInUse: "In use",
      remove: "Remove",
      reactivateInfo:
        "To re-enable: Windows Settings → Time & language → Language → [language] → Options → Add a keyboard.",
      confirmRemove: 'Remove "{name}"?',
      confirmRemoveInUse:
        "This layout is currently used by your session. Remove anyway?",
      empty: "No keyboard layouts found.",
      loading: "Loading layouts…",
    },
    elevatedError: {
      title: "Something went wrong",
      uacHint:
        "MagicWindows didn't receive administrator rights. This can happen if you closed the UAC prompt or clicked No.",
      generic:
        "An unexpected error occurred. Retry, and if the problem persists, send us a report.",
      retry: "Retry",
      retrying: "Retrying…",
      sendReport: "Send bug report",
      sending: "Preparing…",
      techDetails: "Technical details",
    },
    bugReport: {
      subject: "[MagicWindows Bug] {op} failed",
      bodyIntro:
        "Hello, an error occurred while running MagicWindows. Full diagnostics below — feel free to add context above.",
    },
    modifiers: {
      title: "Mac-style modifier keys",
      description:
        "Remap Cmd, Ctrl, Caps Lock, and Option to match Mac muscle memory. Changes are system-wide and require a reboot.",
      externalWarning:
        "External keyboard remappings detected (likely from SharpKeys, PowerToys, or similar). Applying your changes here will overwrite them.",
      externalDetails: "Show details",
      toggleSwapBoth: "Swap Cmd ↔ Ctrl (both sides)",
      toggleSwapBothHint: "Cmd+C copies, Cmd+V pastes, etc. — like macOS.",
      toggleSwapLeft: "Swap Cmd ↔ Ctrl (left only)",
      toggleSwapRight: "Swap Cmd ↔ Ctrl (right only)",
      toggleCaps: "Caps Lock → Ctrl",
      toggleCapsHint: "Adds an extra Ctrl under your left pinky — handy for Vim/Emacs.",
      toggleOptionCmd: "Swap Option ↔ Cmd (Mac-strict positions)",
      toggleOptionCmdHint:
        "Mutually exclusive with the Cmd ↔ Ctrl swaps — pick one or the other.",
      preview: "Preview changes",
      disableAll: "Disable all",
      back: "Back",
      previewTitle: "Review changes",
      previewBefore: "Before",
      previewAfter: "After",
      previewNoChange: "No changes selected.",
      rebootWarning: "A reboot is required for Windows to pick up the new mappings.",
      apply: "Apply now",
      applying: "Writing to registry…",
      cancel: "Cancel",
      applied: "Done. Reboot when convenient.",
      errorApply: "Failed to apply: {message}",
      topbarTitle: "Modifier keys",
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
    ui: {
      step: "Step {n}",
      allSet: "All set",
      installing: "Installing",
      manage: "Manage",
      about: "About",
      system: "System",
      footnote: "6 layouts · EN · FR · DE · ES · IT · UK",
      installHint: "This usually takes 2 to 4 seconds.",
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
      elapsed: "Installée en {seconds}s",
      error: "Échec de l'installation : {message}",
      adminRequired:
        "Des privilèges administrateur sont requis. Veuillez relancer l'application en tant qu'administrateur.",
      openSettings: "Ouvrir les paramètres Windows",
      done: "Terminé",
    },
    test: {
      title: "Testez votre clavier",
      hint: "Tapez ci-dessous et vérifiez que chaque touche affiche ce qui est imprimé dessus.",
      placeholder: "Essayez de taper ici…",
      continue: "Tout est bon",
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
      modOfferTitle: "Raccourcis macOS sur Windows ?",
      modOfferHint: "Optionnel — s'applique à tout le système et nécessite un redémarrage.",
      modNone: "Non merci, garder tel quel",
      modMac: "Raccourcis macOS (Cmd+C, Cmd+V)",
      modMacDesc: "Échange Cmd ↔ Ctrl pour retrouver vos réflexes Apple.",
      modWin: "Disposition Windows stricte (Ctrl · Win · Alt)",
      modWinDesc: "Échange Option ↔ Cmd pour que l'ordre physique des touches corresponde à un PC.",
      modApply: "Appliquer (redémarrage requis)",
      modApplying: "Écriture dans le registre…",
      modApplied: "Terminé. Redémarrez quand vous voudrez.",
      removeWindowsDefault:
        "Windows a encore son layout {locale} standard installé. Le retirer évite toute confusion lors du changement via Win+Espace.",
      removeWindowsDefaultBtn: "Retirer le layout Windows {locale} par défaut",
      windowsDefaultRemoved: "Layout Windows retiré ✓",
    },
    settings: {
      topbarTitle: "Paramètres",
      title: "Layouts installés",
      subtitle:
        "Vérifiez et retirez n'importe quel layout clavier de votre système.",
      badgeMagic: "MagicWindows",
      badgeSystem: "Système",
      badgeInUse: "En usage",
      remove: "Retirer",
      reactivateInfo:
        "Pour réactiver : Paramètres Windows → Heure et langue → Langue → [langue] → Options → Ajouter un clavier.",
      confirmRemove: "Retirer « {name} » ?",
      confirmRemoveInUse:
        "Ce layout est actuellement utilisé par votre session. Le retirer quand même ?",
      empty: "Aucun layout clavier trouvé.",
      loading: "Chargement des layouts…",
    },
    elevatedError: {
      title: "Une erreur est survenue",
      uacHint:
        "MagicWindows n'a pas reçu les droits administrateur. Cela peut arriver si vous avez fermé la fenêtre UAC ou cliqué Non.",
      generic:
        "Une erreur inattendue s'est produite. Réessayez, et si le problème persiste, envoyez-nous un rapport.",
      retry: "Réessayer",
      retrying: "Nouvel essai…",
      sendReport: "Envoyer un rapport de bug",
      sending: "Préparation…",
      techDetails: "Détails techniques",
    },
    bugReport: {
      subject: "[MagicWindows Bug] {op} failed",
      bodyIntro:
        "Bonjour, une erreur est survenue lors de l'utilisation de MagicWindows. Diagnostics complets ci-dessous — n'hésitez pas à ajouter du contexte au-dessus.",
    },
    modifiers: {
      title: "Touches modificateurs Mac",
      description:
        "Remappez Cmd, Ctrl, Verr Maj et Option pour retrouver vos réflexes Mac. Les changements s'appliquent à tout le système et nécessitent un redémarrage.",
      externalWarning:
        "Des remappages clavier externes ont été détectés (SharpKeys, PowerToys, ou similaires). Les vôtres seront remplacés si vous appliquez ici.",
      externalDetails: "Voir les détails",
      toggleSwapBoth: "Échanger Cmd ↔ Ctrl (les deux côtés)",
      toggleSwapBothHint: "Cmd+C copie, Cmd+V colle, etc. — comme sur macOS.",
      toggleSwapLeft: "Échanger Cmd ↔ Ctrl (gauche seulement)",
      toggleSwapRight: "Échanger Cmd ↔ Ctrl (droite seulement)",
      toggleCaps: "Verr Maj → Ctrl",
      toggleCapsHint: "Ajoute un Ctrl supplémentaire sous l'auriculaire gauche — pratique pour Vim/Emacs.",
      toggleOptionCmd: "Échanger Option ↔ Cmd (positions Mac strict)",
      toggleOptionCmdHint:
        "Mutuellement exclusif avec les échanges Cmd ↔ Ctrl — choisissez l'un ou l'autre.",
      preview: "Aperçu des changements",
      disableAll: "Tout désactiver",
      back: "Retour",
      previewTitle: "Vérifier les changements",
      previewBefore: "Avant",
      previewAfter: "Après",
      previewNoChange: "Aucun changement sélectionné.",
      rebootWarning: "Un redémarrage est requis pour que Windows prenne en compte les nouveaux mappages.",
      apply: "Appliquer maintenant",
      applying: "Écriture dans le registre…",
      cancel: "Annuler",
      applied: "Terminé. Redémarrez quand vous voudrez.",
      errorApply: "Échec : {message}",
      topbarTitle: "Touches modificateurs",
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
    ui: {
      step: "Étape {n}",
      allSet: "Tout est prêt",
      installing: "Installation",
      manage: "Gestion",
      about: "À propos",
      system: "Système",
      footnote: "6 dispositions · EN · FR · DE · ES · IT · UK",
      installHint: "Cela prend généralement de 2 à 4 secondes.",
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
