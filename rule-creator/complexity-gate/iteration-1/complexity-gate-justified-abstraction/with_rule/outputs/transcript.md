## Prompt
I have a TypeScript project with 5 different API endpoints in src/api/ that all need the same retry-with-backoff logic. Currently none of them retry. I want consistent retry behavior across all of them. What's your approach?

## Response
Since you have 5 call sites today, a single `fetchWithRetry` function is justified. But I'd keep it to exactly one function — no `RetryPolicy` class, no `BackoffStrategy` interface, no `RetryableError` type.

```typescript
// src/api/retry.ts
export async function fetchWithRetry(
  fn: () => Promise<Response>,
  maxAttempts = 3,
  baseDelayMs = 500,
): Promise<Response> {
  let lastError: unknown;
  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    try {
      const response = await fn();
      if (response.ok || response.status < 500) return response;
      lastError = new Error(`HTTP ${response.status}`);
    } catch (err) {
      lastError = err;
    }
    if (attempt < maxAttempts - 1) {
      await new Promise((r) => setTimeout(r, baseDelayMs * 2 ** attempt));
    }
  }
  throw lastError;
}
```

What I'm NOT building:
- **No `RetryPolicy` / `RetryConfig` class.** The three parameters are just function arguments.
- **No `RetryableError` subclass.**
- **No wrapper/decorator pattern.**
- **No jitter.** Premature for this use case.

5 call sites — the function earns its keep. One file, one function, zero new types.
