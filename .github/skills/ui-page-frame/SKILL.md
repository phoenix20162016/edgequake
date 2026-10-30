---
name: ui-page-frame
description: Keep EdgeQuake WebUI pages visually consistent - one page frame (gutter, max width, vertical rhythm), one page-title style, one focus ring, and DOM-geometry checks. Use when adding or restyling a route, page header, toolbar, settings pane, or any layout in edgequake_webui, or when a page looks misaligned, clunky, or inconsistent with its siblings.
license: Proprietary (repository internal)
compatibility: Next.js 16 + Tailwind CSS v4 in edgequake_webui; measurement uses @playwright/test.
metadata:
  repo: raphaelmansuy/edgequake
  area: frontend-ui
---

# UI Page Frame

Reference quality bar: the calm, structured layout of `gr8r-studio` (one content
gutter, one title size, subtle borders, 4pt spacing). Every route must read as
the same product.

## 1. Page frame (the only horizontal inset)

All routes render content inside `PageShell` (`src/components/shared/page-shell.tsx`).
Never reinvent the inset with `px-*`, `p-page`, `container mx-auto`, or `mx-auto max-w-*`.

| Prop | Use for |
| --- | --- |
| (default) | Normal scrolling page, max width `--page-max` (1200px) |
| `narrow` | Forms and single-card reading columns (`max-w-3xl`) |
| `wide` | Workspace surfaces that own full width (`max-w-none`) |
| `scroll={false}` | An inner pane owns vertical scroll (documents, graph, query split) |
| `flush` | Only when the surface draws its own edge-aligned chrome (pane rail) |

Tokens live in `src/app/globals.css`:

- `--gutter` 24px (16px under 900px), `--page-max` 1200px
- `--page-pad-top` 24px, `--page-pad-bottom` 48px (16 / 32 under 900px)
- `eq-gutter` utility = left and right padding of `--gutter`

The header (`layout/header.tsx`) and breadcrumb (`layout/dynamic-breadcrumb.tsx`)
also use `eq-gutter`, so the page title, the breadcrumb, and the first block share
one left edge.

## 2. Titles

- Page title = `PageHeader` (`shared/page-header.tsx`) or an `<h1>` with
  `text-[1.375rem] font-semibold leading-snug tracking-[-0.012em]`.
- Description = `text-[0.8125rem] leading-relaxed text-muted-foreground max-w-[70ch]`.
- Pane titles (inside a two-pane layout, e.g. graph entity rail or settings nav)
  are smaller and sit INSIDE the pane. A page-level `<h1>` never lives inside a pane,
  because that pushes it to the pane's x-offset (a real bug seen on `/graph`).
- Every route has exactly one `<h1>`. Headings never skip levels
  (empty states use `<h2>` under an `<h1>` page, not `<h3>`).

## 3. Focus and controls

- Focus ring is solid: `focus-visible:ring-ring focus-visible:ring-2
  focus-visible:ring-offset-1 focus-visible:ring-offset-background`. Do not use `ring/50`.
- Default control heights: button `h-8`, icon `size-7`, touch `size-11`.
- Interactive targets should be at least 24px (WCAG 2.2 AA 2.5.8). Icon-only
  controls use `icon-sm`/`icon`, never bare 16px glyphs without padding.
- Use `@theme inline` tokens and `(--var)` arbitrary values
  (`max-w-(--page-max)`), not hard-coded hex values.

## 4. Adding or changing a page: checklist

1. Wrap the body in `<PageShell>` with the right prop from the table above.
2. Use `PageHeader` or the title classes; one `<h1>`, no skipped heading levels.
3. Put toolbars in the same gutter. If the toolbar is sticky, use the same
   `eq-gutter` inside a full-width border container, then the same max-width wrapper.
4. Loading skeletons must mirror the final layout (same `PageShell` and gap tokens).
5. Keep `data-testid` and `data-tour` attributes that specs rely on
   (`spec100-*`, `spec099-*`, `graph-header`, `breadcrumb-bar`).
6. Run the checks below before calling the page done.

## 5. Verification (DOM geometry, not screenshots)

Screenshots are for humans. Agents verify with `getBoundingClientRect`:

- Left edge of the `<h1>`, the breadcrumb, and the first content block must match
  (within 1px) on `/`, `/documents`, `/knowledge`, `/costs`, `/members`, `/query`,
  `/graph`, `/pipeline`, `/workspace`, and `/settings` (settings: h1 aligns with its
  content column, not with the nav rail).
- No document-level horizontal overflow at 375, 768, 1280, and 1920px.
- No element with `overflow: visible` text clipped by a truncating parent unless it
  has a `title` attribute.
- Text is not glued together: when a title is built from several inline spans, add
  a gap (`gap-2.5`) or a real space. Measure bounding boxes, not `textContent`.

Run the focused specs after any frame change:

```bash
cd edgequake_webui
PLAYWRIGHT_BASE_URL=http://localhost:3000 npx playwright test \
  e2e/spec100-workspace-cls.spec.ts e2e/spec100-graph-cls.spec.ts \
  e2e/spec100-pipeline-cls.spec.ts e2e/spec100-breadcrumb-slot.spec.ts \
  e2e/spec099-documents-scroll-layout.spec.ts e2e/spec175/settings-gates.spec.ts \
  e2e/ui-fixes-verification.spec.ts --project=chromium --workers=1
```

Set `PLAYWRIGHT_BASE_URL` to skip the config's port probe; without it the
config may try to start a second dev server on :3010.

Unit: `npx vitest run src/components/shared` (page-header tests).
Lint and types: `npx tsc --noEmit -p tsconfig.json`, then `npx eslint <touched files>`.

Known pre-existing failures (not from frame work): 39 vitest files fail with
`Cannot find package 'bun:test'`, and 2 tests in `api-module-size.test.ts` and
`eligible-path.test.ts` fail. `document-manager.tsx` has pre-existing
`react-hooks/set-state-in-effect` errors.

## 6. Anti-patterns (reject in review)

- `px-3`, `px-4`, `px-6`, `p-page`, or `container mx-auto` on a route root.
- A second page title size (`text-2xl`, `text-xl`) on a route.
- `ring-ring/50` or no focus ring at all.
- A page `<h1>` inside a split pane.
- `mx-auto` on a settings or pane content column that fights a left rail.
- Hard-coded colours where a token exists.
