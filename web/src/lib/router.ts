export type Route =
  | { kind: "home" }
  | { kind: "preview"; layoutId: string }
  | { kind: "desktop" }
  | { kind: "notFound" };

export function parseHash(hash: string): Route {
  // Normalise leading "#" and trailing slashes.
  const raw = hash.replace(/^#/, "").replace(/^\//, "").replace(/\/$/, "");
  if (raw === "" || raw === "home") return { kind: "home" };
  if (raw === "desktop") return { kind: "desktop" };
  const previewMatch = raw.match(/^preview\/([a-z0-9-]+)$/);
  if (previewMatch) return { kind: "preview", layoutId: previewMatch[1] };
  return { kind: "notFound" };
}

export function navigate(target: Route): void {
  const hash = (() => {
    switch (target.kind) {
      case "home":      return "#/";
      case "desktop":   return "#/desktop";
      case "preview":   return `#/preview/${target.layoutId}`;
      case "notFound":  return "#/";
    }
  })();
  if (window.location.hash !== hash) {
    window.location.hash = hash;
  }
}
