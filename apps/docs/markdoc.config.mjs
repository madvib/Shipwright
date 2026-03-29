import { defineMarkdocConfig, component, nodes } from "@astrojs/markdoc/config";

export default defineMarkdocConfig({
  tags: {
    // Tabbed content — show different views (e.g. provider outputs)
    tabs: {
      render: component("./src/components/markdoc/Tabs.astro"),
      attributes: {},
    },
    tab: {
      render: component("./src/components/markdoc/Tab.astro"),
      attributes: {
        label: { type: String, required: true },
      },
    },
    // Callout boxes
    aside: {
      render: component("./src/components/markdoc/Aside.astro"),
      attributes: {
        type: { type: String, default: "note" }, // note | tip | caution | danger
        title: { type: String },
      },
    },
    // Pipeline/flow diagram
    flow: {
      render: component("./src/components/markdoc/Flow.astro"),
      attributes: {},
    },
    step: {
      render: component("./src/components/markdoc/Step.astro"),
      attributes: {
        title: { type: String, required: true },
        icon: { type: String },
      },
    },
    // Card grid for feature highlights
    cards: {
      render: component("./src/components/markdoc/Cards.astro"),
      attributes: {},
    },
    card: {
      render: component("./src/components/markdoc/Card.astro"),
      attributes: {
        title: { type: String, required: true },
        icon: { type: String },
      },
    },
  },
});
