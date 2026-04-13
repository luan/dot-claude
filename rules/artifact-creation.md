# Artifact Creation

Write artifact body content to a temp file with the Write tool, then `cat` it into `ct create`. Heredocs and `echo` corrupt markdown — backticks get escaped to `\`` which breaks rendering.

```bash
# Correct: Write tool creates the file, cat pipes it
cat /tmp/artifact-body.md | ct spec create --topic "..."
rm /tmp/artifact-body.md

# Wrong: heredoc escapes backticks
cat <<'EOF' | ct spec create ...   # backticks become \`
echo "..." | ct spec create ...    # same problem
```
