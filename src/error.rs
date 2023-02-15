#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{context}")]
    Git {
        context: String,
        #[source]
        inner: git2::Error,
    },
    #[error("{context}")]
    Github {
        context: String,
        #[source]
        inner: octocrab::Error,
    },
    #[error("wrong number of remotes: expected 1, have {0}")]
    WrongRemoteCount(usize),
    #[error("inexpressable remote: remote name was not utf-8")]
    InexpressableRemote,
    #[error("remote url not utf-8")]
    RemoteUrlNotUtf8,
    #[error("remote url not recognized as github")]
    RemoteUrlNotGithub,
    #[error("branch name not utf-8")]
    BranchNameNotUtf8,
}

/// Convert a library error into our error type, with context
pub trait ContextErr {
    type Ok;
    fn context<S>(self, s: S) -> Result<Self::Ok, Error>
    where
        S: ToString;
}

impl<T> ContextErr for Result<T, git2::Error> {
    type Ok = T;
    fn context<S>(self, s: S) -> Result<<Self as ContextErr>::Ok, Error>
    where
        S: ToString,
    {
        self.map_err(|inner| Error::Git {
            context: s.to_string(),
            inner,
        })
    }
}

impl<T> ContextErr for Result<T, octocrab::Error> {
    type Ok = T;
    fn context<S>(self, s: S) -> Result<<Self as ContextErr>::Ok, Error>
    where
        S: ToString,
    {
        self.map_err(|inner| Error::Github {
            context: s.to_string(),
            inner,
        })
    }
}

pub fn flatten_errors(err: &dyn std::error::Error) -> String {
    let mut buffer = String::new();
    flatten_errors_inner(err, &mut buffer);
    buffer
}

fn flatten_errors_inner(err: &dyn std::error::Error, buffer: &mut String) {
    buffer.push_str(&err.to_string());
    if let Some(child) = err.source() {
        buffer.push_str(": ");
        flatten_errors_inner(child, buffer);
    }
}
