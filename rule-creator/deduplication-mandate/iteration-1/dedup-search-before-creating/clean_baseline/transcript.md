You already have a `slugify` function in `src/utils/string-helpers.ts:1` — it's re-exported from `src/utils/index.ts`.

Just import it in your blog route:

```ts
import { slugify } from "../utils";
```

No new code needed.
