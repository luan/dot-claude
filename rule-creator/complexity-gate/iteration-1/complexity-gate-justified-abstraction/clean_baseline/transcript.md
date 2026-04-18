The project directory is empty — there's no existing code for me to read.

I can't show you exact code without seeing the actual endpoints. The right approach depends entirely on what's already there:

- **What HTTP client** are you using? (`fetch`, `axios`, `got`, `node-fetch`, etc.)
- **How are the endpoints structured?** (class methods, standalone functions, Express route handlers, etc.)
- **What errors should trigger retry?** (network failures only? 5xx? 429 rate limits?)
- **What backoff parameters?** (max retries, base delay, jitter?)

**General approach I'd take:**

1. **Search for existing retry logic** in the codebase (per deduplication discipline — something may already exist)
2. **Read all 5 endpoint files** to understand the common call pattern
3. **Write one `retryWithBackoff` wrapper** that takes an async operation and retry config
4. **Wrap each endpoint's HTTP call** with it — minimal change per file

Could you either point me to the actual project directory, or share the endpoint files? Then I'll give you the exact code tailored to your setup.
