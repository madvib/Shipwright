import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

export default defineConfig({
  site: "https://docs.getship.dev",
  integrations: [
    starlight({
      title: "Ship Docs",
      social: {},
      sidebar: [
        {
          label: "Getting Started",
          items: [
            { label: "Introduction", slug: "" },
          ],
        },
        {
          label: "Skills",
          autogenerate: { directory: "skills" },
        },
      ],
    }),
  ],
});
