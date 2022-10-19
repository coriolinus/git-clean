# `git-clean`

Clean your repo of excess branches which have been merged upstream.

## Authorization

In the event that you want to use this on a private repo, you will need to authenticate your requests with a token.
You need to create a token at <https://github.com/settings/tokens>. It's not clear what precise scopes are required, but this has been demonstrated to work with a token with both `repo` and `read:org` scopes.

Provide the token with the `--personal-access-token TOKEN` option on the command line. This will cache the token for future use.
