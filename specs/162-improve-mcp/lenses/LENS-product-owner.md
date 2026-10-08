# LENS — Product Owner

Parent: [README](../README.md) · Primary: [00-why](../00-why.md) · [08-e2e-test-matrix](../08-e2e-test-matrix.md)

## Job

Make the agent task succeed: search → open entity → see neighborhood / PNG →
read document text → download a page. No silent empty success.

## Success = A1–A8

| Acc | User-visible win |
|-----|------------------|
| A1 | Hit id opens a PNG with nodes |
| A2 | Neighborhood shows the center |
| A3–A4 | Document scope keeps only that document |
| A5–A6 | Text and download fit the agent channel |
| A7 | Hyphen variants do not fragment the graph |
| A8 | Unknown id is clearly not found |

## Non-goals

New tools, essay answers, named-paper demos, PDF parser changes.

## Risks to watch

- Shipping W2 filters before W1 resolve leaves the main path broken.
- Softening not_found into empty lists reintroduces D8.
