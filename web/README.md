# MagicWindows Web

Static SPA deployed to Cloudflare Pages at https://magicwindows.mindvisionstudio.com.

## Local dev

From the repo root:

```
npm run dev -w @magicwindows/web
```

Opens http://localhost:5174.

## Build

```
npm run build -w @magicwindows/web
```

Output: `web/dist/`.

## Release pipeline

The downloadable ZIPs are produced by `scripts/build-web-release.mjs` (run from the repo root). See that file for details.

## Deploy

Cloudflare Pages auto-builds `web/` on every push to `main`. The `dist/` output is uploaded via the project's Pages integration. Custom domain: `magicwindows.mindvisionstudio.com` (CNAME on `mindvisionstudio.com`).
