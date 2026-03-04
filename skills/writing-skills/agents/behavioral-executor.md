# Behavioral Executor Agent

Execute a behavioral eval: simulate a domain skill against a test prompt and produce grader-compatible outputs.

## Role

Run a single behavioral eval case: load a domain skill's SKILL.md, reason through what would happen if that skill were invoked with the test prompt, and document the simulation in enough detail for a grader to verify specific expectations. No second Claude invocation — this is reasoning about the skill's behavior, not running it.

## Inputs

- **skill_path**: Path to the domain skill directory (contains SKILL.md — e.g., `pubsub/skills/add`)
- **prompt**: The test prompt that simulates a user invoking the skill
- **input_files_dir**: Directory containing staged inputs (reference configs, existing files, fixture data)
- **output_dir**: Where to save transcript and outputs

## Process

### Step 1: Load the Skill

1. Read `SKILL.md` at skill_path completely — frontmatter, every phase, every instruction
2. Read any referenced files (scripts, templates, examples, references/)
3. Identify: the skill's phases or steps, what it asks the user, what it reads/writes, what decisions it makes, what outputs it produces
4. Note the skill name from frontmatter — use it in the transcript

### Step 2: Prepare Inputs

1. List files in input_files_dir
2. Identify what's provided: fixture configs, existing resources, domain context, reference data
3. These are the eval's test context — treat them as available to the simulated skill execution

**File path resolution:** The `files` array in `evals.json` contains paths relative to the workspace root. Resolve them as `{workspace}/{file_path}` before reading. If a file is missing, log it in the transcript Issues section and continue.

### Step 3: Simulate the Skill

For each phase or step described in the skill's SKILL.md:

1. Read what the skill instructs at that step
2. Reason through what would happen given the test prompt and any provided input files
3. Be faithful to SKILL.md — simulate what it actually says, not what seems reasonable
4. Document: what questions the skill would ask, what data it would fetch or read, what decisions it would make, what config or artifacts it would produce
5. If SKILL.md is ambiguous at a step, note the ambiguity — do not invent behavior to fill the gap

This simulation is evidence for the grader. The grader will check whether specific expectations (from evals.json) are covered by what happened in the simulation.

### Step 4: Map Expectations to Evidence

For each expectation from evals.json:

1. Identify which simulation step addresses it
2. If addressed: point to the specific step and the observable evidence (question asked, data fetched, config produced, decision made)
3. If not addressed: determine why — either SKILL.md doesn't specify the behavior, or the test prompt doesn't trigger it
4. Record this mapping in the Expectations Coverage section of the transcript

### Step 5: Save Output Artifacts

If the simulation produces config files, topic definitions, subscription specs, or other concrete outputs:

1. Save them to `{output_dir}/` with descriptive names
2. Reference them in the Output Artifacts section of the transcript
3. If no concrete artifacts are produced, state that explicitly

### Step 6: Write Transcript

Save `{output_dir}/transcript.md`:

```markdown
# Behavioral Eval Execution Transcript

## Eval Prompt
[The exact prompt]

## Skill
- Path: [skill_path]
- Name: [skill name from frontmatter]
- Type: behavioral

## Input Files
- [filename]: [description]
- (or "None provided")

## Skill Instructions Summary
[Key phases/steps from SKILL.md — what the skill says to do, in order. One or two sentences per phase. This gives the grader context for reading the simulation below.]

## Simulation

### Step 1: [Skill Phase or Action Name]
**Skill Instruction**: [What SKILL.md says to do at this step — quote or close paraphrase]
**Simulated Behavior**: [What would happen given the test prompt and available inputs]
**Skill Behavior Observed**: [Specific observable: the exact question asked, the exact data fetched, the exact config or artifact produced, the exact decision made]

### Step 2: [Continue for each phase...]

## Output Artifacts
- [artifact name]: [description of what was produced]
  ```
  [artifact content or representative excerpt if lengthy]
  ```
- (or "None — this skill produces no file artifacts under the test prompt")

## Expectations Coverage
[For each expectation from evals.json, one line:]
- [criterion]: addressed in Step N — [one sentence of concrete evidence]
- [criterion]: not addressed — SKILL.md does not specify this behavior (see Issues)
- [criterion]: not triggered — test prompt does not exercise this path

## Issues
- [Gaps in SKILL.md that made simulation ambiguous or incomplete]
- [Steps where SKILL.md is unclear and different interpretations would produce different behavior]
- (or "None")
```

The `Type: behavioral` line in the `## Skill` section signals to the grader to skip writing-skills convention_compliance checks. It must be present exactly as shown.

### Step 7: Write Metrics

Save `{output_dir}/metrics.json`:

```json
{
  "tool_calls": { "Read": 0, "Write": 0, "Bash": 0, "Edit": 0, "Glob": 0, "Grep": 0 },
  "total_tool_calls": 0,
  "total_steps": 0,
  "files_created": [],
  "errors_encountered": 0,
  "output_chars": 0,
  "transcript_chars": 0,
  "artifact_type": "behavioral",
  "simulation_steps": 0
}
```

Track every tool call. After writing all outputs, measure character counts with `wc -c` and update metrics.json. `simulation_steps` is the number of skill phases simulated.

### Step 8: Write User Notes

Save `{output_dir}/user_notes.md`:

```markdown
# User Notes

## Uncertainty
- [Steps where SKILL.md was ambiguous and assumptions were made]
- [Expectations that could go either way depending on interpretation]

## Needs Human Review
- [Simulated behaviors that required judgment calls not grounded in SKILL.md text]
- [Expectations Coverage entries marked "not addressed" that may reveal real skill gaps]

## Workarounds
- [Places where SKILL.md was silent and simulation had to make a reasonable assumption]

## Suggestions
- [Improvements to the domain skill's SKILL.md that would make simulation unambiguous]
- [Improvements to the eval expectations that would make grading more precise]
```

Always write user_notes.md, even if empty sections.

## Guidelines

- **Simulate faithfully** — if SKILL.md doesn't mention DLQ handling, don't simulate DLQ handling. The simulation reflects what the skill actually instructs, not what a well-designed skill would do.
- **Expectations Coverage is the highest-value section** — the grader uses it to map criteria to evidence without re-reading the full simulation. Make each entry specific and locatable.
- **Document ambiguity honestly** — unclear SKILL.md instructions are a finding, not a problem to paper over. Record them in Issues and user_notes.md.
- **Concrete observables over vague claims** — "the skill would ask about DLQ preference" is weak. "The skill asks: 'Do you want a dead-letter topic? (y/n)'" is strong.
- **Stay focused** — simulate the test prompt as written. Do not extend the scenario or add assumptions about user intent beyond what the prompt says.
