You already have exactly what you need. `src/lib/resilience.ts` exports `withResilience` — it does exponential backoff with configurable max attempts and base delay. It's the same pattern the payment service uses.

For your notification service, just use it the same way `payment.ts` does:

```ts
// src/services/notification.ts
import { withResilience } from '../lib/resilience';

export async function sendEmail(to: string, subject: string, body: string) {
  return withResilience(
    () => fetch('/api/email/send', {
      method: 'POST',
      body: JSON.stringify({ to, subject, body }),
    }),
    { maxAttempts: 4, baseDelayMs: 1000 } // tune to taste for a flaky email API
  );
}
```

No new utility needed — `withResilience` already implements retry with exponential backoff (delays of `baseDelayMs * 2^attempt`: 1s, 2s, 4s with the options above). Bump `maxAttempts` or `baseDelayMs` if the email provider needs more patience.
