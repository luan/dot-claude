#!/usr/bin/env python3
"""Fetch, reply to, and resolve PR review threads.

Works with any GitHub repository - auto-detects repo from current directory.
"""

import argparse
import json
import subprocess
import sys


def get_repo() -> str:
    """Get the current repository in owner/repo format."""
    result = subprocess.run(
        ["gh", "repo", "view", "--json", "owner,name", "-q", ".owner.login + \"/\" + .name"],
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()


def run_gh(args: list[str]) -> str:
    """Run a gh CLI command and return stdout."""
    result = subprocess.run(["gh"] + args, capture_output=True, text=True, check=True)
    return result.stdout


def fetch_unresolved_threads(pr_number: int) -> None:
    """Fetch unresolved review threads from a PR."""
    repo = get_repo()
    owner, repo_name = repo.split("/")
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
            f"repo={repo_name}",
            "-F",
            f"pr={pr_number}",
        ]
    )

    data = json.loads(output)
    threads = (
        data.get("data", {}).get("repository", {}).get("pullRequest", {}).get("reviewThreads", {}).get("nodes", [])
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
    resolved = data.get("data", {}).get("resolveReviewThread", {}).get("thread", {}).get("isResolved", False)

    print(json.dumps({"resolved": resolved}))


def reply_to_comment(pr_number: int, comment_id: int, body: str) -> None:
    """Reply to a review comment."""
    repo = get_repo()
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


def main() -> None:
    parser = argparse.ArgumentParser(description="PR review threads helper")
    parser.add_argument("command", choices=["fetch", "resolve", "reply"], help="Command to run")
    parser.add_argument("--pr", type=int, help="PR number (required for fetch/reply)")
    parser.add_argument("--thread-id", help="Thread node ID (required for resolve)")
    parser.add_argument("--comment-id", type=int, help="Comment database ID (required for reply)")
    parser.add_argument("--body", help="Reply message body (required for reply)")

    args = parser.parse_args()

    if args.command == "fetch":
        if args.pr is None:
            print("Error: --pr required for fetch", file=sys.stderr)
            sys.exit(1)
        fetch_unresolved_threads(args.pr)

    elif args.command == "resolve":
        if args.thread_id is None:
            print("Error: --thread-id required for resolve", file=sys.stderr)
            sys.exit(1)
        resolve_thread(args.thread_id)

    elif args.command == "reply":
        if args.pr is None or args.comment_id is None or args.body is None:
            print("Error: --pr, --comment-id, and --body required for reply", file=sys.stderr)
            sys.exit(1)
        reply_to_comment(args.pr, args.comment_id, args.body)


if __name__ == "__main__":
    main()
