## Prompt
I just spent 2 hours debugging a subtle issue with event bus handlers in useEffect capturing stale closures. Third time someone hit this. What should we do?

## Response
Create a project rule at `<project>/.claude/rules/event-bus-closures.md`:

```markdown
---
paths:
  - src/events/**
  - src/**/*.tsx
  - src/**/*.ts
---

# Event Bus Handlers in React Components

## The Bug
Registering event bus handler via `src/events/registry.ts` inside useEffect captures stale closure state. Subscription doesn't re-register on dep changes.

## Why It's Non-Obvious
The event bus API looks like a standard subscribe/unsubscribe pattern that should work in useEffect. Code appears correct but handler fires with outdated state.

## What To Do Instead
Use a ref to hold the current callback:
```ts
const handlerRef = useRef(callback);
handlerRef.current = callback;
useEffect(() => {
  const unsubscribe = eventBus.on('event', (...args) => handlerRef.current(...args));
  return unsubscribe;
}, []);
```

## Do NOT
- Register event bus handlers directly in useEffect with closure over component state
- Assume adding deps to useEffect dep array fixes it — re-subscribes but drops events in the gap
```

Priority: 1) Write rule file now. 2) Build `useEventBus` hook. 3) Optional lint rule.
