// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// https://astro.build/config
export default defineConfig({
  integrations: [
    starlight({
      title: "qalc docs",
      social: [{ icon: "github", label: "GitHub", href: "https://github.com/guillex/qalc" }],
      sidebar: [
        {
          label: "Introduction",
          items: [
            { label: "What is qalc", slug: "introduction/what-is-qalc" },
            { label: "Functional paradigm", slug: "introduction/functional-paradigm" },
          ],
        },
        {
          label: "Language",
          items: [
            { label: "Basic syntax", slug: "language/basic-syntax" },
            { label: "Function definitions", slug: "language/function-definitions" },
            { label: "Currying", slug: "language/currying" },
            { label: "Function overloading", slug: "language/function-overloading" },
          ],
        },
      ],
    }),
  ],
});
