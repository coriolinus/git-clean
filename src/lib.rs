use std::{ops::Deref, path::Path};

use futures::TryStreamExt;
use git2::{BranchType, Repository};
use lazy_static::lazy_static;
use octocrab::{models::issues::Issue, Octocrab, OctocrabBuilder, Page};
use regex::Regex;
use slog::o;

mod error;
use error::ContextErr;
pub use error::Error;

pub(crate) mod config;
pub mod token;

fn parse_git_url(url: &str) -> Option<(&str, &str)> {
    lazy_static! {
        static ref SSH_RE: Regex =
            Regex::new(r"^git@github.com:(?P<org>\w+)/(?P<repo>\w+).git$").unwrap();
        static ref HTTP_RE: Regex =
            Regex::new(r"^https://github.com/(?P<org>\w+)/(?P<repo>\w+).git$").unwrap();
    }

    let captures = SSH_RE.captures(url).or_else(|| HTTP_RE.captures(url))?;
    let org = captures.name("org")?.as_str();
    let repo = captures.name("repo")?.as_str();

    Some((org, repo))
}

async fn get_pr_page(
    octocrab: impl Deref<Target = Octocrab>,
    owner: &str,
    repo_name: &str,
    branch_name: &str,
    limit: impl Into<Option<u8>>,
) -> Result<Page<Issue>, Error> {
    // Github API specifies a maximum of 100 items returned per page
    let limit = limit.into().unwrap_or(100);

    octocrab
        .search()
        .issues_and_pull_requests(&format!(
            "is:pr repo:{owner}/{repo_name} head:{branch_name}"
        ))
        .per_page(limit)
        .send()
        .await
        .context("search for pull requests by branch")
}

async fn get_prs(
    octocrab: impl Deref<Target = Octocrab>,
    owner: &str,
    repo_name: &str,
    branch_name: &str,
) -> Result<Vec<Issue>, Error> {
    octocrab
        .all_pages(get_pr_page(&*octocrab, owner, repo_name, branch_name, None).await?)
        .await
        .context("get rest of pages for pull requests for a branch")
        .map_err(Into::into)
}

/// Clean up git branches.
///
/// For each local branch, it is in one of these states:
///
///   1. Not pushed to the remote.
///   2. Pushed to the remote but 0 PRs created.
///   3. Pushed to the remote with at least 1 PR created, and at least 1 PR is not closed.
///   4. Pushed to the remote with at least 1 PR created, and all PRs are closed.
///
/// In states 1 - 3, we retain the branch: it is assumed to still be in development.
/// However, in state 4, we delete the branch: it is no longer relevant.
///
/// Closing completed branches helps keep the local dev environment relevant.
pub async fn clean_branches(
    path: impl AsRef<Path>,
    dry_run: bool,
    personal_access_token: Option<String>,
    logger: slog::Logger,
) -> Result<(), Error> {
    let octocrab = {
        let mut builder = OctocrabBuilder::new();
        if let Some(token) = personal_access_token {
            builder = builder.personal_token(token);
        }
        builder.build().context("build octocrab instance")?
    };

    let repo = Repository::open(path).context("open repo from path")?;
    let remotes = repo.remotes().context("list remotes")?;
    if remotes.len() != 1 {
        return Err(Error::WrongRemoteCount(remotes.len()));
    }
    let remote_name = remotes.get(0).ok_or(Error::InexpressableRemote)?;
    let remote = repo
        .find_remote(remote_name)
        .context("get remote by name")?;
    slog::trace!(logger, "got remote"; "name" => remote_name);

    let (owner, repo_name) = parse_git_url(remote.url().ok_or(Error::RemoteUrlNotUtf8)?)
        .ok_or(Error::RemoteUrlNotGithub)?;

    slog::trace!(logger, "parsed url"; "owner" => owner, "repo" => repo_name);

    futures::stream::iter(
        repo.branches(Some(BranchType::Local))
            .context("list local branches")?
            .filter_map(|maybe_branch| maybe_branch.ok())
            .map(|(branch, _branch_type)| Ok::<_, Error>(branch)),
    )
    .try_for_each_concurrent(None, |mut branch| {
        let logger = logger.clone();
        let octocrab = octocrab.clone();

        async move {
            let branch_name = branch
                .name()
                .context("get branch name")?
                .ok_or(Error::BranchNameNotUtf8)?;

            let logger = logger.new(o!("branch name" => branch_name.to_string()));

            let prs = get_prs(&octocrab, owner, repo_name, branch_name).await?;

            slog::trace!(logger, "got prs"; "qty" => prs.len());

            // if there are no prs associated with this branch, then we shouldn't
            // close it; it's local
            if prs.is_empty() {
                return Ok(());
            }

            // otherwise, if all prs associated with this branch are closed, then
            // whether or not they're merged, they're no longer relevant.
            if prs
                .into_iter()
                .any(|pr| !pr.state.eq_ignore_ascii_case("closed"))
            {
                slog::trace!(logger, "retaining branch");
            } else {
                slog::trace!(logger, "deleting branch");
                if !dry_run {
                    branch.delete().context("deleting branch")?;
                }
            }

            Ok(())
        }
    })
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // use std::io::Write;

    use super::*;

    // this can go wrong if someone ever creates another PR with that name
    // in that repo, but for now we'll assume that won't happen
    //
    // We ignore this test by default because it requires a network connection
    // and can be a little slow / use up the API rate limit (60/hr).
    #[tokio::test]
    #[ignore]
    async fn get_pr_by_branch_name() {
        let octocrab = octocrab::instance();

        let page = get_pr_page(octocrab, "coriolinus", "counter-rs", "index", 2)
            .await
            .unwrap();

        let count = page.total_count.unwrap_or_else(|| page.items.len() as _);

        assert_eq!(count, 1);
        assert_eq!(page.items[0].number, 9);
    }
}
