# SvelteKit UI Implementation Rules

> 컴포넌트 층 · 폴더 · 한 줄 판 · 꺼내는 기준은 [`docs/ui-components.md`](docs/ui-components.md)를 따른다 (2026-10-01 사용자 지시). 이 문서와 겹치면 그쪽이 우선한다.

## Scope

These rules apply to pages and UI components built with SvelteKit, Svelte 5, shadcn-svelte, Bits UI, and Tailwind CSS.

Follow the project's existing structure, design tokens, component conventions, and installed package versions. Apply these rules to the requested work. Do not reorganize unrelated code to enforce them.

The goal is code that a developer can read, use, and change without tracing through unnecessary files or abstractions.

## 1. Inspect before creating

Before implementing a UI change:

- Find existing components with the same purpose.
- Check the project's design tokens, custom utilities, and comparable pages.
- Check the installed library and Tailwind versions before using an API or configuration syntax.
- Reuse or extend an appropriate existing component before creating another one.

Do not create a second button, field, card, dialog, or layout pattern solely because a new page needs it.

## 2. Use each UI layer for its purpose

- **shadcn-svelte:** Use the project's installed components as the first choice for standard UI.
- **Bits UI:** Use its primitives when the required interaction is not covered by an existing project component. Do not repeatedly assemble the same primitive pattern in separate pages.
- **Shared UI components:** Provide consistent behavior and appearance across features.
- **Feature components:** Handle a distinct, complete user task within a feature.
- **Pages:** Show the screen's structure in user-facing order and connect data to user actions.

Do not wrap an existing shadcn-svelte component merely to rename it. Create a project wrapper only when it establishes a shared API, behavior, or design rule that is actually used.

**Reason:** Every extra layer increases the number of files a developer must inspect.

## 3. Extract components only with a concrete reason

Create a separate component when at least one condition is met:

1. It is used in at least two real locations.
2. It owns a complete, multi-step user task with its own meaningful state and behavior.

A visual section, card, heading, or table is not automatically a component. Anticipated reuse is not evidence of reuse. File length alone is not a reason to extract every section.

Before extracting, verify that:

- Its responsibility can be described in one sentence.
- Its name and props make its use clear at the call site.
- The resulting page is easier to understand.

Keep a simple, single-use section in its page when extraction fails these checks. Avoid chains of `sections`, `widgets`, `containers`, and `blocks` created to organize unnecessary components.

## 4. Keep component APIs straightforward

- Expose only props required by current call sites.
- Use consistent names and behavior for related controls, including value, size, disabled state, labels, descriptions, and errors.
- Avoid generic components whose behavior depends on combinations of `mode`, `kind`, `type`, `variant`, and configuration objects.
- Do not require callers to repeatedly pass the same bundle of styling classes.
- Keep related controls visually and behaviorally consistent. For example, Select and Combobox in the same form should follow the project's shared field and list conventions.
- Prefer an API that reveals the user task at the call site.

Do not create a Svelte component merely to hide a Tailwind class string. Decide whether a token, custom utility, shared style, or component best represents the repeated pattern.

## 5. Make the page readable in screen order

A developer reading `+page.svelte` from top to bottom should recognize the displayed page: heading, primary actions, filters, content, supporting information, and dialogs.

- Keep simple, single-use markup at its display location.
- Use `{#each}` for data-driven repetition.
- Reduce duplication and misplaced logic before splitting a long page.
- Extract a distinct user task when that makes the page clearer.
- Do not turn the page into a list of opaque components that must all be opened to understand the screen.

There is no fixed line-count limit. Readability and responsibility determine the boundary.

## 6. Use snippets for composition, not page decomposition

Use Svelte snippets for short repeated markup or content passed to a component. Use `child` snippets when required by Bits UI or shadcn-svelte composition.

Do not:

- Move a single-use section into a snippet at the bottom of the page.
- Represent most of a page as a collection of snippet functions.
- Replace a simple `{#if}` or `{#each}` with a snippet.
- Hide a complex user task inside a large snippet to avoid making a proper component.

There is no arbitrary maximum number of snippets. Each snippet must make its call site and displayed markup easier to follow.

## 7. Keep local logic with its component

Put component-specific state, event handlers, short validation, and derived display values in the component's `<script lang="ts">`.

Extract logic only when it:

- Is actually reused;
- Is complex enough to obscure the screen or component;
- Represents an independently maintained business rule; or
- Belongs to an API or server-only boundary.

Do not split `.svelte` and `.ts` files merely to claim that UI and logic are separated. Do not put extensive shared business rules into a page script either.

Assign one owner to each state value. A parent-owned value must not be duplicated as independently authoritative state in a child. A child may own its temporary editing draft.

Do not store per-request or per-user data in server module globals. Do not mutate shared application state from a SvelteKit `load` function.

Clean up subscriptions, timers, observers, and other resources when their owning component no longer uses them.

## 8. Treat Tailwind arbitrary values as exceptions

Do not introduce `[...]` arbitrary values or arbitrary properties as the default way to style new page or feature UI.

Choose styles in this order:

1. Existing shared components and design tokens.
2. Standard Tailwind utilities.
3. A meaningful foundation token for a repeated new value.
4. A shared style, custom utility, or component for a repeated combination of properties.
5. An arbitrary value only when the preceding options do not represent the requirement.

Do not replace `w-[347px]` with a globally named `.width-347` utility. Name shared values by purpose, such as a sidebar width or panel surface.

Use the token and custom-utility syntax supported by the project's installed Tailwind version. Do not assume that all three projects use the same version.

An arbitrary value may be appropriate for a genuinely dynamic value, an exceptional integration requirement, or a one-off value that does not belong in the design system. Explain the reason when adding one.

This restriction applies to newly authored page and feature UI. Do not bulk-rewrite shadcn-svelte source or necessary library composition selectors solely to remove brackets. Review behavior and accessibility before changing library-derived code.

Do not construct Tailwind class names through string interpolation. Use complete class names for conditional styles.

## 9. Preserve user input and async behavior

- Define loading, success, failure, and empty states where the user can encounter them.
- Prevent duplicate save, delete, or upload actions while an operation is pending.
- Preserve entered values when an operation fails; show an actionable error.
- Distinguish persisted data from an editing draft.
- Prevent an older async response from overwriting a newer selection or edit.
- When changing a date, account, tab, route, or dialog would discard an unfinished task, provide appropriate preservation or confirmation behavior. Do not add a confirmation dialog to every navigation by default.

Use the project's established API and submission approach. These rules do not require replacing a Python-backed API flow with SvelteKit form actions.

## 10. Preserve accessibility and responsive behavior

- Use controls with correct semantics, accessible names, keyboard behavior, and visible focus.
- Associate labels, descriptions, and errors with their inputs.
- Check focus behavior when dialogs and popovers open and close.
- Do not suppress Svelte accessibility warnings without a specific, documented reason.
- Check changed screens at the narrow and wide viewport sizes the project supports.
- Inspect tables, long text, dialogs, menus, and empty states for clipping or unusable overflow.

Using Bits UI or shadcn-svelte does not remove the need to check how a composed control works in the actual page.

## 11. Keep files in predictable locations

Follow each project's existing directory structure.

- Put shared UI with shared UI and feature-specific UI with its feature.
- Do not move a component to a shared directory based on possible future reuse.
- Do not add several folder levels for one file.
- Keep code near its owner unless real reuse or a framework boundary requires another location.
- Respect SvelteKit's required route files.

The three projects should share these decision rules; their directory names do not have to be identical.

## 12. Finish within the requested scope

Implement the requested behavior and run the relevant checks already available in the project. Do not perform an unrelated UI migration, general refactor, or design-system expansion.

Before reporting completion, verify:

- The page reads in screen order.
- Every new component meets the extraction criteria.
- Existing UI and design rules are reused.
- Snippets have a clear composition or reuse purpose.
- New arbitrary values have a concrete reason.
- State has one owner and unfinished input is protected where needed.
- Async, accessibility, and responsive states relevant to the change work.
- Subscriptions and other resources are cleaned up.
- The project's relevant checks pass.

Report concisely:

- What changed.
- New components and why they were extracted, if any.
- New tokens, utilities, or arbitrary-value exceptions, if any.
- Checks run and their results.

Do not add a lengthy self-review when these items are sufficient.
