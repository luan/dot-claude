# /plan - Strategic Project Planning

Create comprehensive plans for complex, multi-session projects.

## Usage
`/plan [project description]`

## Purpose
For projects that require:
- Multiple implementation phases
- Architectural decisions
- Cross-system coordination
- Long-term maintenance considerations

## Behavior

### Planning Process
The model will:
1. Assess project complexity and scope
2. Create structured implementation phases
3. Identify key technical decisions
4. Break work into session-sized chunks
5. Set up appropriate memory structures

### Memory Management (Automem)
🧠 Autonomously creates comprehensive planning context using Automem:

**Workflow Creation:**
- `workflow_create` for multi-phase project tracking
- Phase breakdowns with dependencies stored as workflow steps
- `workflow_complete` to mark phase completion
- `workflow_status` to check progress across sessions

**Context Storage:**
- `memory_store` with category "context" for architectural decisions
- `memory_store` with category "observation" for research findings
- `memory_store` with category "action" for implementation plans
- `relationships_store` to link components and decisions

**Task Management:**
- `board_create` for breaking down work into tasks
- `board_move` to track task progress through categories
- `board_status` for visual progress tracking

**Retrieval & Analysis:**
- `memory_search` to find relevant context
- `relationships_traverse` to understand system connections
- `analysis_analyze` for pattern detection

### When to Use
- Building new systems or major features
- Significant refactoring efforts
- Projects spanning multiple sessions
- Work requiring careful coordination

## Examples
```
/plan implement real-time collaborative editing
/plan migrate database from PostgreSQL to MongoDB
/plan build microservices architecture
```

## Philosophy
The model will:
- Create plans that evolve with understanding
- Focus on outcomes over process
- Adapt structure to project needs
- Maintain flexibility for changes