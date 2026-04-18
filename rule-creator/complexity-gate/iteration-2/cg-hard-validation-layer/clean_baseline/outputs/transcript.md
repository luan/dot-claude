Here's what `orders.ts` does:

**`validateCreateOrder(body: unknown)`** takes raw input and returns either `{ data: CreateOrderRequest }` (validated + trimmed) or `{ errors: ValidationError[] }` (all problems at once).

Validation rules:
- **userId** — required, non-empty string
- **items** — required, non-empty array; each element must have a non-empty `productId` string and a `quantity` that's a positive integer
- **couponCode** — optional; if present, must match `/^[a-zA-Z0-9]+$/`

The discriminated union return type means callers check `"errors" in result` or `"data" in result` to branch — no exceptions for expected validation failures. All errors are collected before returning so the client can fix everything in one round trip.
