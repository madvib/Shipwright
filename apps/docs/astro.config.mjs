import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import markdoc from "@astrojs/markdoc";

export default defineConfig({
  site: "https://docs.getship.dev",
  integrations: [
    markdoc(),
    starlight({
      title: "Ship",
      logo: {
        src: "./src/assets/logo.svg",
      },
      social: [],
      customCss: ["./src/styles/ship-theme.css"],
      sidebar: [
        {
          label: "Getting Started",
          autogenerate: { directory: "reference/getting-started" },
        },
        {
          label: "Agents",
          autogenerate: { directory: "reference/agents" },
        },
        {
          label: "Registry",
          autogenerate: { directory: "reference/registry" },
        },
        {
          label: "Studio",
          autogenerate: { directory: "reference/studio" },
        },
        {
          label: "Contributing",
          autogenerate: { directory: "reference/contributing" },
        },
        {
          label: "CLI Reference",
          autogenerate: { directory: "reference/cli" },
        },
        {
          label: "Smart Skills",
          autogenerate: { directory: "reference/smart-skills" },
        },
        {
          label: "MCP Tools",
          autogenerate: { directory: "reference/mcp" },
        },
        {
          label: "Architecture",
          collapsed: true,
          autogenerate: { directory: "reference/architecture" },
        },
        {
          label: "Compiler",
          collapsed: true,
          autogenerate: { directory: "reference/compiler" },
        },
      ],
    }),
  ],
});
