use std::borrow::Cow;

use crate::config::Config;

pub fn save<'a>(
    personal_access_token: impl Into<Cow<'a, str>>,
) -> Result<(), crate::config::Error> {
    let mut config = Config::load().unwrap_or_else(|_| Config::default());
    config.personal_access_token = personal_access_token.into().into_owned();
    config.save()
}

pub fn load() -> Option<String> {
    Config::load()
        .ok()
        .map(|config| config.personal_access_token)
}
