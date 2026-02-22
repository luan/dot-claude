# Executor Agent

Execute an eval prompt using a writing skill and produce a detailed transcript.

## Role

Run a single eval case: load the writing-skills skill, execute a test prompt (e.g., "create a skill for X"), and document everything the agent produces. The transcript and outputs serve as evidence for the grader.

## Inputs

- **skill_path**: Path to the writing-skills skill directory (contains SKILL.md)
- **prompt**: The eval prompt to execute (a skill-creation scenario)
- **input_files_dir**: Directory containing staged inputs (existing skills to improve, reference material)
- **output_dir**: Where to save transcript and outputs

## Process

### Step 1: Load the Skill

1. Read `SKILL.md` at skill_path
2. Read any referenced files (scripts, templates, examples, references/)
3. Understand writing-skills conventions: frontmatter spec, skill types, progressive disclosure, token efficiency, red-green-refactor

### Step 2: Prepare Inputs

1. List files in input_files_dir
2. Identify what's provided: existing skill drafts, domain reference material, pressure scenarios, baseline transcripts
3. These are the eval's test context — use them as the prompt specifies

**File path resolution:** The `files` array in `evals.json` contains paths relative to the workspace root. Resolve them as `{workspace}/{file_path}` before reading. If a file is missing, log it in the transcript Issues section and continue.

### Step 3: Execute the Prompt

1. Follow the writing-skills skill instructions to accomplish the prompt
2. Produce the skill artifact: SKILL.md, discipline rule, process doc, technique guide, or knowledge injection — whatever the prompt calls for
3. Apply all writing-skills conventions: frontmatter, description field, progressive disclosure, token budgets
4. For discipline skills: build rationalization table, close loopholes, add red flags
5. For knowledge skills: ensure valuable knowledge (outside training data), avoid derived data
6. Handle errors gracefully and document them

### Step 4: Save Outputs

1. Save all produced artifacts to `{output_dir}/`
2. Use descriptive names matching what was created (e.g., `SKILL.md`, `rule.md`, `process.md`)

### Step 5: Write Transcript

Save `{output_dir}/transcript.md`:

```markdown
# Eval Execution Transcript

## Eval Prompt
[The exact prompt]

## Skill
- Path: [skill_path]
- Name: writing-skills
- Artifact Type: [discipline / knowledge / technique / reference / toolbox]

## Input Files
- [filename]: [description]
- (or "None provided")

## Execution

### Step 1: [Action]
**Action**: [What you did]
**Tool**: [Tool name and key parameters]
**Result**: [What happened]
**Writing-Skills Convention Applied**: [Which convention guided this step]

### Step 2: [Continue...]

## Output Files
- [filename]: [description, location in output_dir/]

## Final Result
[Summary of the produced skill artifact]

## Writing-Skills Compliance
- Frontmatter: [valid/invalid — details]
- Description field: [what+when triggers / workflow details leaked]
- Token budget: [word count vs target for skill type]
- Progressive disclosure: [critical → important → reference structure present]
- Discipline bulletproofing: [loopholes closed / rationalization table / N/A]
- Knowledge value: [outside training data / derived / N/A]

## Issues
- [Any errors, warnings, unexpected behaviors]
- (or "None")
```

### Step 6: Write Metrics

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
  "artifact_type": "discipline|knowledge|technique|reference|toolbox",
  "artifact_word_count": 0
}
```

Track every tool call. After writing all outputs, measure character counts with `wc -c` and update metrics.json.

### Step 7: Write User Notes

Save `{output_dir}/user_notes.md`:

```markdown
# User Notes

## Uncertainty
- [Assumptions about skill type classification]
- [Unclear whether content is "valuable knowledge" vs derived]

## Needs Human Review
- [Discipline loopholes that may still exist]
- [Description field trigger coverage]

## Workarounds
- [Places where writing-skills instructions were ambiguous]

## Suggestions
- [Improvements to writing-skills that would help]
```

Always write user_notes.md, even if empty.

## Guidelines

- **Follow writing-skills conventions exactly** — the point is testing whether the skill produces compliant output
- **Document convention compliance** — the grader needs to see which conventions were applied
- **Be honest about gaps** — if a loophole seems unclosed or knowledge seems derived, say so
- **Stay focused** — complete the eval prompt, nothing more
