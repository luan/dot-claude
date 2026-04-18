Good question. This is a known footgun with any imperative subscription API + React closures. Three occurrences means it's an API design problem, not a user education problem.

**Root cause**: The event bus `register()` presumably takes a callback and holds it. In `useEffect`, that callback closes over render-scoped state. If the effect deps don't include every captured variable — or worse, the effect doesn't re-register on change — the handler sees stale values forever.

**Two complementary fixes, in priority order:**

### 1. `useEventBusHandler` hook (eliminates the class of bug)

The ref-stable-callback pattern — same idea behind React's `useEffectEvent` proposal:

```ts
function useEventBusHandler(event: string, handler: (...args: any[]) => void) {
  const handlerRef = useRef(handler);
  handlerRef.current = handler; // always points to latest closure

  useEffect(() => {
    const stableHandler = (...args: any[]) => handlerRef.current(...args);
    const unsub = eventBus.register(event, stableHandler);
    return unsub; // cleanup on unmount
  }, [event]); // subscribes once per event name, but always calls fresh handler
}
```

This makes it impossible to capture stale state — the ref always points to the latest closure, and the subscription is stable (no churn). Callers just write:

```ts
useEventBusHandler('some-event', (payload) => {
  // freely use current state, props, anything — always fresh
  setState(prev => prev + payload.value);
});
```

### 2. Lint rule or CLAUDE.md rule to flag raw `eventBus.register` inside `useEffect`

Once the hook exists, direct usage of the registry inside effects is a code smell. An ESLint rule (or a project rule for code review) that flags `register(` inside `useEffect` callbacks pushes people toward the safe path.

**The hook is the real fix.** Education and rules help, but if the unsafe API is still the easiest thing to reach for, people will keep hitting this. Make the safe path the default path.

Want me to look at `src/events/registry.ts` and implement the hook?
