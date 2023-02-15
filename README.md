# `git-clean`

Clean your repo of excess branches which have been merged upstream.

There are five possible cases for a local branch:

1. It is the default branch (`main`/`master`)
2. Not pushed to the remote.
3. Pushed to the remote but 0 PRs created.
4. Pushed to the remote with at least 1 PR created, and at least 1 PR is not closed.
5. Pushed to the remote with at least 1 PR created, and all PRs are closed.

In cases 1 - 4, we retain the branch: it is assumed to still be in development or otherwise relevant.
However, in state 5, we delete the branch: it is no longer relevant.

## Authorization

In the event that you want to use this on a private repo, you will need to authenticate your requests with a token.

You need to create a classic token at <https://github.com/settings/tokens> with at least the permissions `repo` and `read:org`.

Provide the token with the `--personal-access-token TOKEN` option on the command line. This will cache the token for future use.
