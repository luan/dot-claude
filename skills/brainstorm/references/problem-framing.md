# Problem Framing — Challenge Patterns

Patterns to recognize during the interview challenge gate. When input matches a pattern, surface the question before proceeding.

## Solution-shaped inputs

User describes a thing to build without stating what problem it solves.

| Pattern | Example | Why it fails | Push back with |
|---------|---------|--------------|----------------|
| Named artifact without stated problem | "Build a CLI tool", "Add a caching layer" | Skips the problem entirely — the solution may solve the wrong thing | "What breaks today without this?" |
| Architecture decision as a starting point | "Migrate to microservices", "Rewrite in Go" | Locks in approach before validating the need | "Who's blocked and on what?" |
| Feature request with no user or context | "Add dark mode", "Support multi-tenancy" | No signal on urgency, scope, or whether it's the right lever | "What user pain does this address?" |

## Unvalidated assumptions

User states things as fact without evidence.

| Pattern | Example | Why it fails | Push back with |
|---------|---------|--------------|----------------|
| Claimed user desire | "Users want faster load times", "Teams need a dashboard" | May reflect internal belief, not observed behavior | "Have we confirmed this with users or data?" |
| Claimed system limitation | "The system cannot handle concurrent requests", "Our DB won't scale" | May be untested or based on outdated knowledge | "Has this been measured or is it a hypothesis?" |
| Assumed consensus | "Everyone agrees we need to refactor this", "The team already decided" | Consensus claims often mask unresolved disagreement | "Who signed off, and what was the decision context?" |

## Weak problem statements

Too broad, too vague, or describing a symptom rather than a cause.

| Pattern | Example | Why it fails | Push back with |
|---------|---------|--------------|----------------|
| Vague quality complaint | "DX is bad", "The API feels messy" | No actionable scope — could mean anything | "Which specific workflow? For whom? How often?" |
| Symptom without cause | "Things are slow", "Deployments keep failing" | Identifies an effect, not a root cause | "Where exactly? Under what conditions?" |
| Modernization framing | "We need to modernize", "This is legacy" | Age is not a problem — impact is | "What does this prevent you from doing today?" |
