// @ts-check
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";

// https://astro.build/config
export default defineConfig({
  integrations: [
    starlight({
      title: "Unity Catalog Rust",
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/unitycatalog-incubator/unitycatalog-rs",
        },
      ],
      sidebar: [
        {
          label: "Tutorials",
          autogenerate: { directory: "tutorials" },
        },
        {
          label: "Guides",
          autogenerate: { directory: "guides" },
        },
        {
          label: "Explanation",
          autogenerate: { directory: "explanation" },
        },
        {
          label: "Reference",
          autogenerate: { directory: "reference" },
        },
      ],
      plugins: [],
    }),
  ],
});
