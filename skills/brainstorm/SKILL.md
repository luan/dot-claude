---
name: brainstorm
description: "Collaborative design for greenfield features and new ideas. Triggers: 'brainstorm', 'ideate', 'new feature design', 'help me think through', 'what should we build', 'help me design', 'think through X with me', 'I want to build something new'."
argument-hint: "<idea or topic> [--auto]"
user-invocable: true
allowed-tools:
  - Agent
  - Bash
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - WebSearch
  - WebFetch
  - AskUserQuestion
---

# Brainstorm

Turn vague ideas into actionable designs through collaborative dialogue.
For vision/exploratory work, produces structured vault output (hub + linked deep dives).

**Main thread only.** Interactive dialogue stays here; context scanning and research use background subagents.

## Hard Gate

Present and get approval on a design before any implementation. "Simple" projects are where unexamined assumptions waste the most work.

## Instructions

### 1. Scan Project Context + Start Interview

Dispatch up to 2 background Explore agents in parallel:
- **Code scan**: tech stack, relevant patterns, adjacent code, constraints. Under 30 lines.
- **Documentation scan**: project vault, shared docs, existing specs/spikes related to the topic. Read key documents directly -- summaries from subagents are not enough for architectural decisions.

Empty/new project -- skip scan, ask stack preferences in interview.

Begin the interview immediately -- scan results feed Step 3.

### 2. Interview

`--auto` -> skip interview. Infer purpose, scope, constraints, and success criteria from prompt + project context.

Without `--auto`: AskUserQuestion, ONE per turn. Prefer multiple-choice.

**Skip interview only if** the prompt has ALL three: explicit scope boundaries (non-goals stated), measurable constraints, and testable success criteria. Acknowledge by citing 2+ concrete details. When in doubt, interview.

**Sequence** (adapt, skip irrelevant):

1. **Purpose** -- What problem? Who's it for?
2. **Scope** -- Minimum useful version? (YAGNI gate)
3. **Constraints** -- Performance, compatibility, security, timeline?
4. **Prior art** -- Similar code in codebase or elsewhere?
5. **Success criteria** -- How will you know it works?

Stop when you can propose approaches. Usually 3-5 questions, never >7.

**Mid-dialogue pivot:** If direction shifts fundamentally, acknowledge, discard stale context, restart from the relevant question.

### 3. Industry Research (conditional)

**Trigger:** The topic involves architecture decisions, product direction, ecosystem integration, or greenfield design where external context matters. Skip for purely internal work (refactoring, bug triage, isolated features).

Dispatch 1-2 research subagents using WebSearch/WebFetch with specific, targeted questions -- not broad surveys. Good: "Who ships QUIC in production game servers? Name projects and cite sources." Bad: "Research QUIC for games."

- Industry landscape: what exists, what's shipping, what's vaporware
- Prior art and competing approaches
- Ecosystem standards and benchmarks relevant to the topic

Keep research focused. Max 800 words per agent. Present findings as grounding for approaches, not as a literature review. If the first round is insufficient, dispatch a second round with more targeted questions based on what was missing.

### 4. Propose 2-3 Approaches

Check that both scans completed. Read key documents yourself if the scan surfaced important ones -- don't rely solely on subagent summaries for architectural decisions. Incorporate findings from code scan, documentation, and industry research.

Lead with recommendation + justification referencing user's constraints. Non-recommended: 2-3 sentences + downside vs recommended. Be opinionated.

**Independent thinking:** User inputs during the interview are hypotheses, not conclusions. If research contradicts an assumption the user stated, surface it. Don't echo the user's framing as the thesis -- build your own from evidence.

`--auto` -> auto-select the recommended approach. Without `--auto` -> ask user to pick or refine. All rejected -> ask what's missing, propose new approaches.

### 5. Present Design (iterative)

Scale to complexity. `--auto` -> skip per-section confirmations. Without `--auto` -> confirm after each section.

Include only relevant sections: architecture, data flow, API surface, error handling, testing.

**Clarify as you go.** If the design introduces concepts the user may not know (protocols, algorithms, infrastructure patterns), explain them inline. The brainstorm should educate while designing.

**Iteration rounds.** After presenting the full design, ask for feedback with structured options:
- **Direction feedback** (thesis/framing needs adjustment) -> re-enter from Step 4
- **Content feedback** (details/examples need work) -> revise specific sections
- **Approved** -> proceed to Step 6

Handle granular pushback: the user may accept some sections and push back on others. Address each point individually rather than re-presenting the entire design. If pushback raises new research questions, dispatch targeted research agents before responding.

On major pushback, don't restart from scratch. Re-enter the relevant phase. Track which round you're in (Round 1/2/3) so the user sees progress.

**Honest maturity.** Not every topic deserves equal depth. Flag what's speculative vs. proven. If you don't have evidence for a claim, say so rather than presenting it with false confidence.

### 6. Output + Direction

**For simple brainstorms** (isolated features, small scope):

```
Brainstorm: <topic>
Problem: <1 sentence>
Approach: <1 sentence>
Next: /spec
```

The design lives in this conversation. When ready to formalize, run `/spec`.

**For vision/exploratory brainstorms** (architecture, product direction, multi-faceted design):

When the conversation has matured and the user is satisfied with the direction, offer to write structured output to the project's shared vault.

**Output structure:**
- **Hub document** -- one-pager: problem, thesis/principles (brief), current position, actionable direction, `[[wiki-links]]` to deep dives
- **Linked deep dives** -- one per major concept, independently readable, linked back to hub with `Part of [[hub-name]]`

**Location:** Project's shared vault or documentation directory. Ask the user where to write if not obvious from project context. Use `ct` if the project uses the blueprints system.

**Naming:** `YYYYMMDD-<issue>-<slug>.md` for hub, `<slug>-<subtopic>.md` for deep dives.

**Frontmatter:** Each file gets:
```yaml
---
topic: <topic>
created: <date>
author: <user>
tags:
  - type/spec
  - status/needs-review
  - project/<project>
---
```

**Actionable direction section** (in the hub):
- **Now** -- concrete actionables with ticket-ready titles, not vague TODOs
- **Next** -- what to do once preconditions are met
- **Then/Later** -- long-horizon items with honest maturity assessment

End with `Next:` chain: could be `/spec` for implementation, `/develop` for execution, or "share with team for review."

## Key Principles

- **YAGNI:** Push back on scope creep during interview
- **Independent thinking:** User inputs are hypotheses. Research, reason, push back. Don't echo the user's framing as the thesis.
- **Honest maturity:** Flag what's speculative vs. proven. Not every section deserves equal depth.
- **Industry grounding:** Claims about ecosystem/standards must be backed by research, not assumptions. Use WebSearch when the topic warrants it.
- **Obsidian-native output:** `[[wiki-links]]`, proper frontmatter, no periods on bold titles in vault docs.
- **Design is the deliverable** -- implementation details belong in the spec
