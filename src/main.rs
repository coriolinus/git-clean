use clap::Parser;
use color_eyre::Result;
use git_clean::{clean_branches, token};
use slog::Logger;

fn slog_init() -> Logger {
    use slog::o;
    use slog::Drain;

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    slog::Logger::root(drain, o!())
}

/// Clean outdated local git branches.
///
/// Removes local branches which have been pushed to the remote, and at least 1
/// PR has been created for them, and all such PRs are now closed.
#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    /// Use and cache a GitHub Personal Access Token
    ///
    /// This must be a "classic" token and it must have at least
    /// `repo` and `read:org` permissions assigned.
    ///
    /// To create a token, visit
    /// <https://github.com/settings/tokens>.
    #[arg(long, short = 'T')]
    personal_access_token: Option<String>,

    /// Do not actually edit the repository.
    #[arg(short, long)]
    dry_run: bool,

    /// Path to the repository to clean
    #[arg(default_value = ".")]
    path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let logger = slog_init();
    let args = Args::parse();

    if let Some(token) = args.personal_access_token {
        token::save(token)?;
    }

    clean_branches(args.path, args.dry_run, token::load(&logger), logger).await?;
    Ok(())
}
