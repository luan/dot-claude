#!/usr/bin/env python3
"""Stop hook for non-interactive mode continuation.

Uses AI to evaluate if current session's work is complete.
"""

import json
import os
import subprocess
import sys

# Configuration
NON_INTERACTIVE_ENV = "CLAUDE_NON_INTERACTIVE"
CONTEXT_THRESHOLD_PERCENT = 85
MIN_CONTEXT_FOR_TASK = 50000


def check_non_interactive_mode() -> bool:
    return os.environ.get(NON_INTERACTIVE_ENV, "").lower() in ("1", "true", "yes")


def get_context_info(hook_input: dict) -> dict:
    used = hook_input.get("context_window_used", 0)
    total = hook_input.get("context_window_total", 200000)
    if total == 0:
        total = 200000
    percent_used = (used / total) * 100 if total > 0 else 0
    remaining = total - used
    return {
        "percent_used": percent_used,
        "remaining": remaining,
        "can_continue": percent_used < CONTEXT_THRESHOLD_PERCENT
        and remaining > MIN_CONTEXT_FOR_TASK,
    }


def get_recent_transcript(hook_input: dict, max_chars: int = 8000) -> str:
    """Extract recent conversation from transcript."""
    transcript_path = hook_input.get("transcript_path", "")
    if not transcript_path or not os.path.exists(transcript_path):
        return ""

    try:
        messages = []
        with open(transcript_path, "r") as f:
            for line in f:
                try:
                    entry = json.loads(line)
                    role = entry.get("role", "")
                    if role in ("user", "assistant"):
                        content = entry.get("message", {}).get("content", [])
                        texts = [
                            c.get("text", "")
                            for c in content
                            if c.get("type") == "text"
                        ]
                        if texts:
                            messages.append(f"{role}: {' '.join(texts)}")
                except json.JSONDecodeError:
                    continue

        # Take last N messages that fit in max_chars
        result = ""
        for msg in reversed(messages[-10:]):
            if len(result) + len(msg) > max_chars:
                break
            result = msg + "\n\n" + result
        return result.strip()
    except Exception:
        return ""


def get_current_issue_context() -> str:
    """Get current issue info for context."""
    try:
        result = subprocess.run(
            ["bd", "list", "--status", "in_progress", "--json"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        if result.returncode == 0:
            issues = json.loads(result.stdout)
            if issues:
                issue = issues[0]
                return f"Current issue: {issue.get('id')} - {issue.get('title', 'untitled')}\nNotes excerpt: {issue.get('notes', '')[:1000]}"
    except Exception:
        pass
    return "No in_progress issue found."


def evaluate_with_ai(transcript: str, issue_context: str) -> dict:
    """Use AI to evaluate if session work is complete."""
    prompt = f"""Evaluate if this coding session's work appears COMPLETE or INCOMPLETE.

{issue_context}

Recent conversation:
{transcript}

Respond with ONLY a JSON object:
{{"complete": true/false, "reason": "brief explanation", "next_action": "what to do if incomplete"}}

Signs of COMPLETE:
- Explicitly closed issue (bd close)
- Called finishing-branch
- Said "done", "complete", "finished"
- No more tasks mentioned

Signs of INCOMPLETE:
- Mid-task, mentioned "next" or "continuing"
- Error occurred, needs fixing
- Tests failing
- Explicitly said more work needed

If unclear, default to COMPLETE (don't loop forever)."""

    try:
        result = subprocess.run(
            ["claude", "-p", prompt],
            capture_output=True,
            text=True,
            timeout=30,
            env={
                **os.environ,
                "CLAUDE_NON_INTERACTIVE": "",
            },  # Don't inherit non-interactive
        )

        # Log for debugging
        debug_log = f"/tmp/stop-hook-debug-{os.getpid()}.log"
        with open(debug_log, "w") as f:
            f.write(f"returncode: {result.returncode}\n")
            f.write(f"stdout: {result.stdout[:2000]}\n")
            f.write(f"stderr: {result.stderr[:2000]}\n")

        if result.returncode == 0 and result.stdout.strip():
            response = result.stdout.strip()
            start = response.find("{")
            end = response.rfind("}") + 1
            if start >= 0 and end > start:
                return json.loads(response[start:end])
            # If no JSON found, try to interpret the response
            lower = response.lower()
            if "incomplete" in lower or "not complete" in lower:
                return {
                    "complete": False,
                    "reason": "AI said incomplete",
                    "next_action": "Continue work",
                }
            return {"complete": True, "reason": "AI said complete", "next_action": ""}

        return {
            "complete": True,
            "reason": f"CLI failed: rc={result.returncode}, stderr={result.stderr[:100]}",
            "next_action": "",
        }
    except Exception as e:
        return {
            "complete": True,
            "reason": f"exception: {str(e)[:100]}",
            "next_action": "",
        }


def evaluate_continuation(hook_input: dict) -> dict:
    """Main evaluation: should we continue this session's work?"""
    if not check_non_interactive_mode():
        return {"decision": "approve"}

    context = get_context_info(hook_input)
    transcript = get_recent_transcript(hook_input)
    issue_context = get_current_issue_context()

    # Use AI to evaluate
    evaluation = evaluate_with_ai(transcript, issue_context)

    # Build status
    status_parts = [
        f"context: {context['percent_used']:.0f}%",
        f"eval: {evaluation.get('reason', 'unknown')[:50]}",
    ]
    status_str = ", ".join(status_parts)

    # If session appears complete, allow stop
    if evaluation.get("complete", True):
        return {
            "decision": "approve",
            "continue": False,
            "stopReason": "session_complete",
            "systemMessage": f"✅ Non-interactive complete. [{status_str}]",
        }

    # Session incomplete - check if we CAN continue
    if not context["can_continue"]:
        return {
            "decision": "approve",
            "continue": False,
            "stopReason": "context_full",
            "systemMessage": f"⚠️ Context {context['percent_used']:.0f}% full. [{status_str}]",
        }

    # Session incomplete and context available - continue
    next_action = evaluation.get("next_action", "Continue with remaining work.")
    return {
        "decision": "block",
        "reason": next_action,
        "systemMessage": f"🔄 Continuing. [{status_str}]",
    }


def main():
    try:
        hook_input = json.load(sys.stdin)

        # Log hook input to debug
        with open("/tmp/stop-hook-input.json", "w") as f:
            json.dump(hook_input, f, indent=2, default=str)

        result = evaluate_continuation(hook_input)
        print(json.dumps(result))
    except Exception as e:
        print(json.dumps({"decision": "approve", "systemMessage": f"Hook error: {e}"}))
    finally:
        sys.exit(0)


if __name__ == "__main__":
    main()
