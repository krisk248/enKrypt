import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { VitePWA } from 'vite-plugin-pwa';

// Strict Content-Security-Policy injected into the built index.html.
//  - `script-src 'wasm-unsafe-eval'` permits WebAssembly compilation without
//    allowing arbitrary JS eval.
//  - `connect-src 'self'` allows fetching the local .wasm/manifest only — no
//    external hosts, no telemetry, no keyservers.
//  - `style-src 'unsafe-inline'` is required by Svelte scoped styles / inline
//    style attributes; no external stylesheets are ever loaded.
const CSP = [
  "default-src 'self'",
  "base-uri 'self'",
  "object-src 'none'",
  "frame-ancestors 'none'",
  "form-action 'none'",
  "img-src 'self' data:",
  "font-src 'self'",
  "style-src 'self' 'unsafe-inline'",
  "script-src 'self' 'wasm-unsafe-eval'",
  "connect-src 'self'",
  "worker-src 'self' blob:",
  "manifest-src 'self'",
].join('; ');

function cspPlugin() {
  return {
    name: 'enkrypt-csp',
    apply: 'build' as const,
    transformIndexHtml(html: string) {
      return html.replace(
        '</title>',
        `</title>\n    <meta http-equiv="Content-Security-Policy" content="${CSP}" />`,
      );
    },
  };
}

// enKrypt is a fully self-contained, offline-first static app. No external hosts.
export default defineConfig({
  plugins: [
    cspPlugin(),
    svelte(),
    wasm(),
    topLevelAwait(),
    VitePWA({
      registerType: 'autoUpdate',
      // Precache every built asset (incl. the .wasm) so the app runs offline.
      workbox: {
        globPatterns: ['**/*.{js,css,html,wasm,svg,png,ico,woff2}'],
        maximumFileSizeToCacheInBytes: 8 * 1024 * 1024,
        navigateFallback: 'index.html',
      },
      includeAssets: ['favicon.svg'],
      manifest: {
        name: 'enKrypt — OpenPGP Privacy Tray',
        short_name: 'enKrypt',
        description:
          'Client-side OpenPGP file encryption. Keys and files never leave your machine.',
        theme_color: '#0ea5e9',
        background_color: '#0b1220',
        display: 'standalone',
        start_url: '.',
        scope: '.',
        icons: [
          { src: 'icon-192.png', sizes: '192x192', type: 'image/png' },
          { src: 'icon-512.png', sizes: '512x512', type: 'image/png' },
          {
            src: 'icon-512.png',
            sizes: '512x512',
            type: 'image/png',
            purpose: 'maskable',
          },
        ],
      },
    }),
  ],
  worker: {
    format: 'es',
    plugins: () => [wasm(), topLevelAwait()],
  },
  // Keep the wasm out of inline base64 so it can be precached as its own asset.
  build: {
    target: 'esnext',
    assetsInlineLimit: 0,
  },
  server: { port: 5173 },
});
