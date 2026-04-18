You already have a rate limiter — `src/middleware/throttle.ts` does exactly this: tracks request counts per key within a sliding window and rejects when the limit is exceeded.

Instead of creating a new `src/middleware/rate-limit.ts`, wrap the existing `isThrottled` in an Express middleware and apply it to your router:

```ts
// src/middleware/rate-limit.ts
import { Request, Response, NextFunction } from 'express';
import { isThrottled } from './throttle';

const WINDOW_MS = 60_000;
const MAX_REQUESTS = 100;

export function rateLimit(req: Request, res: Response, next: NextFunction): void {
  const key = req.ip ?? 'unknown';
  if (isThrottled(key, MAX_REQUESTS, WINDOW_MS)) {
    res.status(429).json({ error: 'Too many requests' });
    return;
  }
  next();
}
```

Then in `src/routes/api.ts`:

```ts
import { rateLimit } from '../middleware/rate-limit';

router.use(rateLimit);
```

That's it. The `isThrottled` function already handles window expiry and count tracking. No need to duplicate that logic.

**One caveat:** the in-memory `Map` in `throttle.ts` won't survive restarts and doesn't work across multiple server instances. If you're running multiple processes/containers, you'd want to swap the backing store for Redis (e.g., `ioredis` with a simple `INCR`/`EXPIRE` pair). But for a single-process server, this is fine.

Want me to create the file?
