import manifestData from "../../public/manifest.json";

export type Arch = "x64" | "arm64";
export const ARCHES: Arch[] = ["x64", "arm64"];

export const ARCH_LABEL: Record<Arch, string> = {
  x64: "x64",
  arm64: "ARM64",
};

export interface ManifestDownload {
  url: string;
  size: number;
  sha256: string;
}

export type LayoutDownloads = Partial<Record<Arch, ManifestDownload>>;

export interface Manifest {
  version: string;
  generatedAt: string;
  downloads: Record<string, LayoutDownloads>;
}

function isDownloadEntry(value: unknown): value is ManifestDownload {
  if (!value || typeof value !== "object") return false;
  const d = value as Record<string, unknown>;
  return typeof d.url === "string"
    && typeof d.size === "number"
    && typeof d.sha256 === "string";
}

export function isValidManifest(value: unknown): value is Manifest {
  if (!value || typeof value !== "object") return false;
  const m = value as Record<string, unknown>;
  if (typeof m.version !== "string") return false;
  if (typeof m.generatedAt !== "string") return false;
  if (!m.downloads || typeof m.downloads !== "object") return false;
  for (const [, layoutEntry] of Object.entries(m.downloads as Record<string, unknown>)) {
    if (!layoutEntry || typeof layoutEntry !== "object") return false;
    for (const [arch, dl] of Object.entries(layoutEntry as Record<string, unknown>)) {
      if (arch !== "x64" && arch !== "arm64") return false;
      if (!isDownloadEntry(dl)) return false;
    }
  }
  return true;
}

export const manifest: Manifest = isValidManifest(manifestData)
  ? (manifestData as Manifest)
  : { version: "0.0.0", generatedAt: "1970-01-01T00:00:00Z", downloads: {} };

/** Returns all arch-specific download entries for a layout, or empty object if unknown. */
export function downloadsFor(layoutId: string): LayoutDownloads {
  return manifest.downloads[layoutId] ?? {};
}
