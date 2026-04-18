## Prompt
I have a TypeScript project. I need to add retry logic to a single API call in src/api/client.ts. The function `fetchUserProfile` currently makes one fetch call. Add retry with 3 attempts and exponential backoff. Show me the code you'd write.

## Response
The straightforward approach: inline the retry loop directly in `fetchUserProfile`. No retry utility, no wrapper, no abstraction — there's only one call site.

```typescript
async function fetchUserProfile(userId: string): Promise<UserProfile> {
  const maxAttempts = 3;
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      const response = await fetch(`/api/users/${userId}/profile`);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      return await response.json();
    } catch (error) {
      if (attempt === maxAttempts) {
        throw error;
      }
      await new Promise((resolve) => setTimeout(resolve, 2 ** attempt * 100));
    }
  }
  throw new Error("unreachable");
}
```

Key decisions:
- **No `RetryPolicy` class, no `withRetry` wrapper, no `BackoffStrategy` interface.** There is one call site. Inlining is ~10 lines of straightforward loop logic.
- **Exponential backoff**: 200ms, 400ms before the 2nd and 3rd attempts.
- **Retries on both network errors (fetch throws) and HTTP errors (non-2xx).**
