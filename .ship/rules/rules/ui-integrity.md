# UI Integrity

* Never display fake data. If a count is 0, show 0 or hide the element. Aspirational numbers are lies.

* Never render a button without a working handler. If the capability doesn't exist, don't show the button. Use an orange dot indicator (w-2 h-2 rounded-full bg-primary) only for buttons that are structurally correct but need backend wiring.

* Don't broadcast capabilities that don't exist yet. Hide stats, sections, and features until they have real data behind them. An empty state is better than a fake one.

* Design specs define information architecture, not just visuals. When a spec says "3 dock items" that means replace the existing dock, not add pages to the old one.

* Composition is the deliverable. Individual components are worthless if they aren't wired into the navigation flow. The dock, nav, page transitions, and data flow between pages define the product.

* Use the project's design tokens (styles.css, @ship/primitives, Tailwind config). Never hardcode colors, fonts, or spacing. Map design intent to existing tokens.

* The approved design spec at docs/specs/ is the source of truth for UI work. Read it before touching apps/web/.
