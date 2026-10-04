# LENS — UX / UI Designer (SPEC-159)

Parent: [README](../README.md) · Spec: [05](../05-ux-ui-spec.md)

## Job

Make Ask the obvious next step when inspecting a node or page, with a calm
landing: evidence beside an editable question, human in control of send.

## Journey map

```text
  Explore graph / read PDF
        |
        v
  Intent: "I want EdgeQuake's take on THIS"
        |
        v
  Ask about this / Ask about this page
        |
        v
  Land: new chat + evidence pane + seed focused
        |
        +--> edit question (optional)
        +--> Send
        v
  Answer + citations (existing Query UX)
```

## Heuristics

| Heuristic | Application |
|-----------|-------------|
| Recognition over recall | Seed includes label/title/page already seen |
| User control | No auto-send |
| Consistency | Same Ask pattern as in-query “Ask about this” |
| Error visibility | Pane empty/404 copy, not silent blank |
| Match real world | “Page” means the page you see |

## Content rules

- Prefer human labels over ids.
- Never claim page-exclusive search.
- Toast optional and short; do not block composer.

## A11y bar

- Menuitem and header button have accessible names (i18n).
- Keyboard reaches Ask without pointer.
- Focus lands in composer after navigation.
- en/fr/zh parity for new strings.

## Cross-ref

US-159-01…04 · LAW-159-2,9 · EC-159-05,10,13 · OPP-159-01…04
