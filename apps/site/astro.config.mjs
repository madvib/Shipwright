import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
import cloudflare from '@astrojs/cloudflare';
import tailwindcss from "@tailwindcss/vite";

import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  adapter: cloudflare({
    platformProxy: {
      enabled: true
    },
    imageService: "cloudflare"
  }),
  integrations: [react()],
  output: 'static',
  vite: {
    plugins: [tailwindcss()],
    resolve: {
      alias: {
        '@ui': fileURLToPath(new URL('../crates/ui/src', import.meta.url)),
        '@': fileURLToPath(new URL('../crates/ui/src', import.meta.url))
      }
    }

  }
});
