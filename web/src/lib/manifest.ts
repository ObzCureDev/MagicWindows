import manifestData from "../../public/manifest.json";

export interface ManifestDownload {
  url: string;
  size: number;
  sha256: string;
}

export interface Manifest {
  version: string;
  generatedAt: string;
  downloads: Record<string, ManifestDownload>;
}

export function isValidManifest(value: unknown): value is Manifest {
  if (!value || typeof value !== "object") return false;
  const m = value as Record<string, unknown>;
  if (typeof m.version !== "string") return false;
  if (typeof m.generatedAt !== "string") return false;
  if (!m.downloads || typeof m.downloads !== "object") return false;
  for (const [, dl] of Object.entries(m.downloads as Record<string, unknown>)) {
    if (!dl || typeof dl !== "object") return false;
    const d = dl as Record<string, unknown>;
    if (typeof d.url !== "string") return false;
    if (typeof d.size !== "number") return false;
    if (typeof d.sha256 !== "string") return false;
  }
  return true;
}

export const manifest: Manifest = isValidManifest(manifestData)
  ? (manifestData as Manifest)
  : { version: "0.0.0", generatedAt: "1970-01-01T00:00:00Z", downloads: {} };

export function downloadFor(layoutId: string): ManifestDownload | null {
  return manifest.downloads[layoutId] ?? null;
}
