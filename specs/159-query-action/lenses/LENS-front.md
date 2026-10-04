# LENS — Front Designer (SPEC-159)

Parent: [README](../README.md) · UX: [05](../05-ux-ui-spec.md) · Frontend: [06](../06-frontend-architecture.md)

## Job

Specify visual placement, density, and component reuse so Ask feels native to
Graph Studio and Document detail — not a bolted-on experiment.

## Visual system

| Token / component | Use |
|-------------------|-----|
| Radix `DropdownMenuItem` | Graph context Ask |
| Existing header `Button` variant | Document Ask |
| Lucide `MessageSquareText` | Icon consistency |
| Type color dot | Keep menu header as-is |
| Companion shell | Unchanged chrome; new entity empty/error via `PaneNotice` |

## Layout constraints

```text
  Document header actions (md+)
  [Download][Ask][Graph][…]  — single row; icon+label

  Document header (narrow)
  [Download][Ask icon][Graph icon] — tooltip + aria-label
```

Graph menu: +1 item; rely on Radix collision padding (already 8).

## States to design

| State | Treatment |
|-------|-----------|
| Default | Standard item/button |
| Loading neighborhood | PaneNotice spinner copy |
| 404 entity | PaneNotice error |
| Companion off | No pane; chat full width |
| Seed ready | Composer filled; Send enabled when non-empty |

## Do not

- Introduce a modal wizard for Ask.
- Use destructive styling for Ask.
- Animate the whole page on handoff beyond normal route change.

## Cross-ref

LAW-159-2,5,9 · EC-159-08,13 · [05](../05-ux-ui-spec.md)
