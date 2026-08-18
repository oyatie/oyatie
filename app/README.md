# app/ — composition glue (`integ/app`)

**OVERRULE 3d:** durable multi-month products do **not** live on this rail.

| Product forever home | Product integ rail | Tip (2026-08-10) |
|----------------------|--------------------|------------------|
| `app/translate/**` | `integ/translate` | `298560ff9` |
| `app/sites/**` | `integ/sites` | `318d2f95a` |
| `app/docs/**` | `integ/app-docs` (not plane `integ/docs`) | `9a2ed96e1` |
| `app/hr/**` | `integ/hr` | `abdff0399` |

This tip owns **composition / glue** under `app/**` that is not claimed by a product rail.
Product absorbs write only on `integ/<product>` with envelope `app/<product>/**`.
