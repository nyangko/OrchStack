# SvelteKit UI and Component Rules

Applies to SvelteKit, Svelte 5, shadcn-svelte, Bits UI, and Tailwind CSS projects.

**Primary rule:** A developer should be able to read a page from top to bottom, understand its screen and user actions, and change it without tracing unnecessary files.

Follow the repository's existing instructions, installed versions, design system, and issue contract. Apply these rules within the requested task. Do not start an unrelated migration.

## This project (OrchStack)

- `{project}` = `orch` → `src/lib/components/orch/`.
- `ui/` holds **shadcn-svelte components only** (사용자 지시 2026-10-01). Product-independent components that shadcn does not have (Kanban, Combobox, Checklist, …) also live under `orch/`, built as generic as possible.
- Callers use **named tags only** — `<KanbanBoard><KanbanColumn><KanbanColumnHeader>`, `<DropdownMenu><DropdownMenuTrigger>` — not `import * as X` / `<X.Root>`. Each `index.ts` exports every part as `<Component><Part>`.
- Canonical stylesheet: `src/app.css` (raw values in `:root` / `.dark`, mappings in `@theme inline`). Role styles: `src/lib/styles/roles.css`. Styles owned by one screen with real repeated use: `src/lib/styles/screens.css`.

---

## 1. Inspect before creating

Before writing or modifying UI:

- Inspect the page and comparable screens.
- Find existing UI and project components with the same purpose.
- Find the canonical tokens, component styles, utilities, variants, and class-merging conventions.
- Check installed versions before using framework or Tailwind syntax.
- Identify who owns the data, editing draft, and submission.
- Reuse or extend a suitable component before creating another.

## 2. Component boundaries

| Layer | Typical location | Responsibility |
|---|---|---|
| Foundation | The project's canonical stylesheet | Theme values, tokens, and shared styles |
| UI primitives | `src/lib/components/ui/` | Product-independent controls |
| Project components | `src/lib/components/{project}/` | This product's concepts, workflows, and complex UI |
| Layout | SvelteKit layouts or existing layout components | Shared application shell |
| Page | `src/routes/**` | Screen order, composition, data, and user actions |
| Shared logic and API | Existing project locations | Reused rules, contracts, and server communication |

Use the actual project name in place of `{project}`. Keep existing repository paths when they already express these boundaries.

A component belongs in `ui/` only when its public API and behavior are independent of this product's concepts. Reuse within one product does not make it a universal UI primitive.

For example, a board designed for OrchStack tasks, statuses, and agent workflows belongs under `components/orch/`, even if several OrchStack pages use it. Its independent Button or Popover controls remain in `ui/`.

**Reason:** Directory placement communicates whether a component carries product assumptions.

## 3. Use libraries and wrappers deliberately

- Prefer an installed shadcn-svelte component for a standard control.
- Use Bits UI when an existing project component does not provide the required interaction.
- Do not rebuild the same Bits UI composition in multiple pages.
- Do not wrap a component merely to rename its import.
- Create a wrapper when it establishes a shared API, behavior, or design rule used by real call sites.
- Provide a ready-to-use API when callers repeatedly assemble the same structure. Keep lower-level composition available for exceptional cases. Do not require a "single" wrapper for every shadcn-svelte component.

## 4. Extract components only with a concrete reason

Create a separate component when at least one condition applies:

1. It has at least two actual call sites.
2. It owns a complete user task with meaningful state and behavior.
3. It is an independent complex UI, such as a viewer, editor, chart, or advanced input.

A visual card, heading, section, or table is not automatically a component. File length and possible future reuse are not sufficient reasons.

Before extracting, confirm that its responsibility fits in one sentence, its name and props explain its use, and the page becomes easier to read.

An extracted project dialog should normally own its frame, form state, and submission flow. Do not leave the frame in the page while hiding only its inner fields in another component.

Do not create `sections/`, `widgets/`, or `containers/` merely to organize unnecessary components. Group project components further only when the existing files warrant it.

## 5. Make pages readable in display order

A `+page.svelte` file should show the screen in approximately the order users encounter it: heading, primary actions, filters, content, supporting information, and dialogs.

- Keep simple, single-use markup at its display location.
- Use data and `{#each}` for repeated structures.
- Remove duplication and misplaced logic before splitting a long page.
- Extract a complete task or independent complex UI when it improves readability.
- Do not replace the screen with an opaque list of tiny components.

There is no automatic line-count threshold for extraction.

## 6. Use snippets for composition

Use snippets for required library composition, child content, or short markup genuinely reused within a file.

Do not move a single-use section away from its display position, represent most of a page as snippet functions, or replace ordinary data-driven rendering with snippets.

Bits UI and shadcn-svelte `child` snippets remain valid when their APIs require them.

## 7. Keep local state and behavior together

Put component-specific state, event handlers, short validation, and display calculations in that component's `<script lang="ts">`.

Extract logic when it is reused, independently maintained, too complex for the component to remain readable, or belongs to an API or server boundary. Do not create a `.ts` file merely to separate "logic" from markup.

Give each value one owner. A child may own an editing draft, but parent and child must not keep independent authoritative copies of the same value.

A project component may coordinate its own pending state, submission, and notification when these complete its user task. Keep API transport and shared business rules in their established locations.

Do not store per-user or per-request data in server module globals. Clean up subscriptions, timers, observers, and similar resources when they are no longer needed.

## 8. Design APIs for callers

- Expose only props needed by current call sites.
- Keep related controls consistent in value, size, disabled state, labels, descriptions, and errors.
- Use variants for a small number of meaningful choices.
- Avoid overlapping `mode`, `kind`, `type`, `variant`, and configuration props.
- Preserve internal interaction when accepting caller event handlers.
- Preserve library attributes, keyboard behavior, and focus handling.
- Do not make callers pass the same long class list repeatedly.
- Use compound components such as Root, Column, and Item when flexible composition is genuinely needed. Do not force every component into that structure.

---

## Foundation, Classes, and Tokens

## 9. Use one canonical source for each design role

Locate the project's canonical stylesheet before adding tokens. Its path may differ across projects; do not assume it is always `src/routes/layout.css`.

One design role has one source value. For example, the card surface has one `--card` value. Do not add `--surface` or another independently maintained value for the same role.

For Tailwind CSS v4, organize the foundation as:

- **Raw values:** Theme-dependent values in `:root` and the project's dark-theme selector.
- **Tailwind mappings:** `@theme` names that expose raw values to utilities where individual utility access is useful.
- **Role styles:** Classes such as `card` that apply a complete, coherent default appearance.

A Tailwind mapping may require a differently named CSS variable to connect a raw value to a utility. That mapping is a technical alias, not a second independent design decision.

Use syntax supported by the installed Tailwind version. Do not apply v4 directives to a project using another version.

### Example: card foundation in Tailwind CSS v4

```css
:root {
  --card: #ffffff;
  --card-foreground: #111827;
  --card-border: rgb(20 23 28 / 0.07);
  --card-shadow:
    0 1px 2px -1px rgb(20 23 28 / 0.06),
    0 2px 6px rgb(20 23 28 / 0.05);
}

.dark {
  --card: #171a1f;
  --card-foreground: #e8eaee;
  --card-border: rgb(255 255 255 / 0.1);
  --card-shadow: none;
}

@theme inline {
  --color-card: var(--card);
  --color-card-foreground: var(--card-foreground);
  --color-card-border: var(--card-border);
  --shadow-card: var(--card-shadow);
  --radius-card: 12px;
}

/* A card surface for elements that do not need the Card component API. */
@utility card {
  background-color: var(--card);
  color: var(--card-foreground);
  border: 1px solid var(--card-border);
  border-radius: var(--radius-card);
  box-shadow: var(--card-shadow);
}
```

Check for an existing `card` class before registering this utility. If that name is already owned by another style, reconcile the existing definition rather than defining a competing `.card`.

The project's `<Card>` component and the `card` class must use the same foundation values. Prefer reusing the same class where it fits. Do not maintain a second set of card colors, radius, border, or shadow in the component.

### Card usage

```svelte
<div class="card p-4">Default card surface</div>
<div class="card bg-black text-white p-4">Intentionally black card</div>
```

`card` supplies the default appearance. An explicit utility may replace an individual property when the design calls for it. Verify the generated result and class-merging behavior in the installed Tailwind setup.

`bg-black` means fixed black in both themes. If the override must change between light and dark themes, use an existing semantic color or define a new role; do not assume `bg-black` will switch automatically.

Do not write `bg-card rounded-card border-card-border shadow-card` at every card call site merely to recreate the card role.

**Reason:** A role class supplies a complete default. A caller only states what differs.

## 10. Choose styles in this order

1. Existing component and its variant.
2. Existing role class such as `card`.
3. Standard Tailwind utilities and existing semantic tokens.
4. A new token for a repeated design value.
5. A named shared style or custom utility for a repeated property pattern.
6. An arbitrary value only when the preceding choices do not represent the requirement.

Do not convert `w-[347px]` into `.width-347`. Name shared values by purpose, such as sidebar width or dialog maximum height.

Do not use the number of classes as an extraction threshold. Create a named style when its combined appearance has a stable meaning and real use.

## 11. Define utilities and variants by purpose

- Use a custom utility for a small reusable behavior or a coherent role style. Use a component variant for a supported appearance of that component.
- Check for collisions with Tailwind utilities, shadcn-svelte styles, and existing project classes.
- Do not define two classes with the same role and different values.
- Keep product statuses out of generic UI variant names.
- Map a product status to its label and visual tone at the project boundary.
- Use foundation values inside variants, not raw hex or palette colors.
- Use the project's established variant mechanism. If it already uses tailwind-variants, follow its `tv()` conventions; do not introduce the package solely for this rule.
- A custom utility's intent should be clear from its name. Add a short comment when the reason or cascade behavior is not obvious.
- Scope overrides for an external renderer, map, drawing engine, or viewer under one owning class. Map supported library CSS variables to foundation values where possible. Explain unusual layer overrides or `!important`.

## 12. Limit arbitrary values and inline styles

Do not use `[...]` arbitrary values as the default styling method in newly authored pages and project components.

Exceptions include a genuinely one-off layout value, an external integration requirement, or a precise value that cannot be represented by the existing foundation.

Use a `style` property or CSS variable for values computed at runtime, such as positions or proportions. Do not generate Tailwind class names from runtime values.

Explain a non-obvious arbitrary-value exception near its use or in the change report. Do not bulk-edit shadcn-svelte selectors such as `data-[state=open]:` merely to eliminate brackets.

## 13. Compose complete class names

Do not build Tailwind class names through string interpolation.

```svelte
<!-- Avoid -->
<span class="bg-{tone}-soft text-{size}">

<!-- Use complete classes -->
<span class={remaining < 0 ? 'text-destructive' : 'text-primary'}>
```

For several choices, select from complete class strings or a defined variant.

Use the project's `cn()` helper where merging is part of a component's API, commonly in `ui/`. Do not use it throughout pages to compensate for conflicting class bundles. A project component may use it when it intentionally supports caller class overrides.

A component's `class` prop is primarily for placement, width, and local layout. If callers repeatedly replace its colors, type scale, or internal appearance, improve its default role style or add a meaningful variant.

Do not create a Svelte component solely to hide classes.

## 14. Keep theme behavior in the foundation

When a semantic role has light and dark values, redefine its raw value under the project's dark-theme selector. Do not scatter color-only `dark:` classes across pages and feature components for that role.

A fixed-color illustration or intentionally fixed `bg-black` override is different: it remains fixed unless the design explicitly defines another theme behavior.

Do not introduce raw hex or RGB colors into newly authored `.svelte` UI. Do not use palette colors for product statuses when semantic status roles exist. Do not add page-level `<style>` blocks merely to hold design values.

---

## Interaction and Delivery

## 15. Preserve input and async behavior

- Show relevant pending, success, error, and empty states.
- Prevent duplicate submissions while an operation is pending.
- Preserve entered values after failure and show an actionable error.
- Distinguish persisted data from an editing draft.
- Prevent an older response from overwriting a newer selection or edit.
- Protect unfinished work when changing a date, account, tab, route, or dialog would discard it.

Use the project's existing API and submission architecture. This document does not mandate SvelteKit form actions or a new API layer.

## 16. Preserve accessibility and responsive behavior

- Use semantic controls with accessible names and visible focus.
- Associate labels, descriptions, and errors with inputs.
- Preserve keyboard and focus behavior when composing Bits UI or shadcn-svelte components.
- Do not suppress Svelte accessibility warnings without a specific reason.
- Check changed screens at supported narrow and wide viewport sizes.
- Inspect tables, long content, menus, dialogs, and empty states for clipping.
- Check relevant states in both light and dark themes when both are supported.

## 17. Verify proportionally and stay in scope

Run relevant type, lint, build, and focused tests already available in the repository. Inspect the actual screen in a browser when layout or interaction changes.

For a visual refactor intended to preserve appearance, compare representative before and after states at consistent viewport and data settings. Do not require a full screenshot suite for every routine UI task.

Do not start an unrelated component migration, folder reorganization, design-system expansion, or issue hierarchy. Check the staged diff before committing, particularly when another session may share the worktree.

Before completing, confirm:

- The page reads in display order.
- Each new component meets an extraction criterion and is in the correct layer.
- A project-specific workflow has not been placed in `ui/`.
- One design role has one source value.
- Role classes provide complete defaults; callers specify only differences.
- Explicit override utilities actually win in the project's generated CSS.
- Arbitrary values have a concrete reason.
- Class names are complete rather than dynamically assembled.
- State has one owner and unfinished input is protected.
- Relevant accessibility, viewport, and theme states have been checked.

Report concisely:

- Changed screens and behavior.
- New components and their extraction reasons.
- New tokens, role styles, utilities, variants, or arbitrary-value exceptions.
- Relevant browser and code checks with results.
- A concrete blocker or unresolved decision, if one remains.
