use color_eyre::Result;
use ezcli::{flag, option};
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

#[tokio::main]
async fn main() -> Result<()> {
    if flag!(-h, --help) {
        let my_name = std::env::current_exe()
            .ok()
            .and_then(|path| {
                path.file_name()
                    .map(|filename| filename.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| "git-clean".into());

        println!(
            "usage: {my_name} [--personal-access-token TOKEN] [--path path_to_repo] [--dry-run]"
        );
    } else {
        color_eyre::install()?;
        let logger = slog_init();

        if let Some(token) = option!(--personal_access_token) {
            token::save(token)?;
        }

        let path = option!(-p, --path).unwrap_or_else(|| String::from("."));
        clean_branches(path, flag!(-d, --dry_run), token::load(), logger).await?;
    }
    Ok(())
}
