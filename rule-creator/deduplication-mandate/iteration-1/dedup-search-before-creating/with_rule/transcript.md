## Prompt
Slugify function needed for src/routes/blog.ts. Project at /tmp/dedup-test with existing src/utils/.

## Response
**A `slugify` function already exists.** No new code needed.

Found at /tmp/dedup-test/src/utils/string-helpers.ts, re-exported from /tmp/dedup-test/src/utils/index.ts.

Existing implementation:
```ts
export function slugify(text: string): string {
  return text.toLowerCase().trim().replace(/[^\w\s-]/g, "").replace(/[\s_-]+/g, "-").replace(/^-+|-+$/g, "");
}
```

In `src/routes/blog.ts`, just import it:
```ts
import { slugify } from "../utils";
```
