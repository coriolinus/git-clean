use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub personal_access_token: String,
}

impl Config {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .expect("platorm has a config dir")
            .join("git-clean.toml")
    }

    pub fn save(&self) -> Result<(), Error> {
        self.save_at(Self::path())
    }

    pub fn save_at(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        use std::io::Write;

        let serialized = toml::to_string_pretty(self).context("serialize config")?;
        let mut file = std::fs::File::create(path).context("create config file")?;
        writeln!(file, "{serialized}").context("write config to file")?;
        Ok(())
    }

    pub fn load() -> Result<Self, Error> {
        Self::load_at(Self::path())
    }

    pub fn load_at(path: impl AsRef<Path>) -> Result<Self, Error> {
        let data = std::fs::read_to_string(path).context("read config data from file")?;
        toml::from_str(&data).context("deserialize config file")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{context}")]
    TomlSerialize {
        context: String,
        #[source]
        inner: toml::ser::Error,
    },
    #[error("{context}")]
    TomlDeserialize {
        context: String,
        #[source]
        inner: toml::de::Error,
    },
    #[error("{context}")]
    Io {
        context: String,
        #[source]
        inner: std::io::Error,
    },
}

trait WithContext {
    type Ok;
    fn context(self, s: impl ToString) -> Result<Self::Ok, Error>;
}

impl<T> WithContext for Result<T, toml::ser::Error> {
    type Ok = T;
    fn context(self, s: impl ToString) -> Result<T, Error> {
        self.map_err(|inner| Error::TomlSerialize {
            context: s.to_string(),
            inner,
        })
    }
}

impl<T> WithContext for Result<T, toml::de::Error> {
    type Ok = T;

    fn context(self, s: impl ToString) -> Result<T, Error> {
        self.map_err(|inner| Error::TomlDeserialize {
            context: s.to_string(),
            inner,
        })
    }
}

impl<T> WithContext for Result<T, std::io::Error> {
    type Ok = T;

    fn context(self, s: impl ToString) -> Result<T, Error> {
        self.map_err(|inner| Error::Io {
            context: s.to_string(),
            inner,
        })
    }
}
