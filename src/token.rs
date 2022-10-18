use std::borrow::Cow;

use slog::Logger;

use crate::config::Config;

pub fn save<'a>(
    personal_access_token: impl Into<Cow<'a, str>>,
) -> Result<(), crate::config::Error> {
    let mut config = Config::load().unwrap_or_else(|_| Config::default());
    config.personal_access_token = personal_access_token.into().into_owned();
    config.save()
}

pub fn load(logger: &Logger) -> Option<String> {
    Config::load()
        .map_err(|err| {
            slog::warn!(logger, "failed to load configuration file"; "err" => err.to_string());
            err
        })
        .ok()
        .map(|config| config.personal_access_token)
}
