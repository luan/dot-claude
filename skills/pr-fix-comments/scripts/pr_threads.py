#!/usr/bin/env python3
"""Fetch, reply to, and resolve PR review threads."""

import argparse
import json
import subprocess
import sys


def run_gh(args: list[str]) -> str:
    """Run a gh CLI command and return stdout."""
    result = subprocess.run(["gh"] + args, capture_output=True, text=True, check=True)
    return result.stdout


def detect_repo() -> str:
    """Auto-detect repo from current working directory."""
    try:
        output = run_gh(
            ["repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"]
        )
        return output.strip()
    except subprocess.CalledProcessError:
        print(
            "Error: could not detect repo. Run from inside a git repo.", file=sys.stderr
        )
        sys.exit(1)


def fetch_unresolved_threads(repo: str, pr_number: int) -> None:
    """Fetch unresolved review threads from a PR."""
    owner, name = repo.split("/")
    query = """
    query($owner: String!, $repo: String!, $pr: Int!) {
      repository(owner: $owner, name: $repo) {
        pullRequest(number: $pr) {
          reviewThreads(first: 100) {
            nodes {
              id
              isResolved
              path
              line
              comments(first: 10) {
                nodes {
                  id
                  databaseId
                  author { login }
                  body
                }
              }
            }
          }
        }
      }
    }
    """

    output = run_gh(
        [
            "api",
            "graphql",
            "-f",
            f"query={query}",
            "-f",
            f"owner={owner}",
            "-f",
            f"repo={name}",
            "-F",
            f"pr={pr_number}",
        ]
    )

    data = json.loads(output)
    threads = (
        data.get("data", {})
        .get("repository", {})
        .get("pullRequest", {})
        .get("reviewThreads", {})
        .get("nodes", [])
    )

    unresolved = []
    for thread in threads:
        if thread.get("isResolved"):
            continue

        comments = thread.get("comments", {}).get("nodes", [])
        if not comments:
            continue

        unresolved.append(
            {
                "thread_id": thread.get("id"),
                "path": thread.get("path"),
                "line": thread.get("line"),
                "comments": [
                    {
                        "id": c.get("id"),
                        "database_id": c.get("databaseId"),
                        "author": c.get("author", {}).get("login"),
                        "body": c.get("body"),
                    }
                    for c in comments
                ],
            }
        )

    print(json.dumps({"unresolved_threads": unresolved}, indent=2))


def resolve_thread(thread_id: str) -> None:
    """Resolve a review thread."""
    query = """
    mutation($threadId: ID!) {
      resolveReviewThread(input: {threadId: $threadId}) {
        thread { isResolved }
      }
    }
    """

    output = run_gh(
        [
            "api",
            "graphql",
            "-f",
            f"query={query}",
            "-f",
            f"threadId={thread_id}",
        ]
    )

    data = json.loads(output)
    resolved = (
        data.get("data", {})
        .get("resolveReviewThread", {})
        .get("thread", {})
        .get("isResolved", False)
    )

    print(json.dumps({"resolved": resolved}))


def reply_to_comment(repo: str, pr_number: int, comment_id: int, body: str) -> None:
    """Reply to a review comment."""
    output = run_gh(
        [
            "api",
            f"repos/{repo}/pulls/{pr_number}/comments",
            "-f",
            f"body={body}",
            "-F",
            f"in_reply_to={comment_id}",
        ]
    )

    data = json.loads(output)
    print(
        json.dumps(
            {
                "replied": True,
                "comment_id": data.get("id"),
                "url": data.get("html_url"),
            }
        )
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="PR review threads helper")
    sub = parser.add_subparsers(dest="command", required=True)

    fetch_p = sub.add_parser("fetch", help="Fetch unresolved threads")
    fetch_p.add_argument("--pr", type=int, required=True, help="PR number")

    resolve_p = sub.add_parser("resolve", help="Resolve a review thread")
    resolve_p.add_argument("--thread-id", required=True, help="Thread node ID")

    reply_p = sub.add_parser("reply", help="Reply to a review comment")
    reply_p.add_argument("--pr", type=int, required=True, help="PR number")
    reply_p.add_argument(
        "--comment-id", type=int, required=True, help="Comment database ID"
    )
    reply_p.add_argument("--body", required=True, help="Reply message body")

    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()
    repo = detect_repo()

    if args.command == "fetch":
        fetch_unresolved_threads(repo, args.pr)
    elif args.command == "resolve":
        resolve_thread(args.thread_id)
    elif args.command == "reply":
        reply_to_comment(repo, args.pr, args.comment_id, args.body)


if __name__ == "__main__":
    main()
