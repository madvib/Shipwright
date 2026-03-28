import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
export default defineConfig({
  site: "https://docs.getship.dev",
  integrations: [
    starlight({
      title: "Ship Docs",
      social: [],
      sidebar: [
        { label: "Introduction", link: "/" },
        {
          label: "CLI",
          autogenerate: { directory: "reference/cli" },
        },
        {
          label: "MCP",
          autogenerate: { directory: "reference/mcp" },
        },
        {
          label: "Compiler",
          autogenerate: { directory: "reference/compiler" },
        },
        {
          label: "Smart Skills",
          autogenerate: { directory: "reference/smart-skills" },
        },
        {
          label: "Studio",
          autogenerate: { directory: "reference/studio" },
        },
      ],
    }),
  ],
});
