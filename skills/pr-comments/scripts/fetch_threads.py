#!/usr/bin/env -S uv run --script
"""Fetch unresolved PR review threads."""

import argparse
import json
import subprocess
import sys


def run_gh(args: list[str]) -> str:
    result = subprocess.run(["gh"] + args, capture_output=True, text=True, check=True)
    return result.stdout


def detect_repo() -> str:
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


def main() -> None:
    parser = argparse.ArgumentParser(description="Fetch unresolved PR review threads")
    parser.add_argument("--pr", type=int, required=True, help="PR number")
    parser.add_argument("--repo", type=str, help="owner/name (default: detect from cwd)")
    args = parser.parse_args()
    repo = args.repo or detect_repo()
    fetch_unresolved_threads(repo, args.pr)


if __name__ == "__main__":
    main()
