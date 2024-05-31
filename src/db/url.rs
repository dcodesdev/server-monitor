use std::{fmt::Display, ops::Deref};

#[derive(Debug, Clone)]
pub struct Url(String);

impl Url {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn strip_prefix(&self) -> &str {
        let url = self.as_str();

        if url.starts_with("http://") {
            &url[7..]
        } else if url.starts_with("https://") {
            &url[8..]
        } else {
            url
        }
    }
}

impl From<String> for Url {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Deref for Url {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
