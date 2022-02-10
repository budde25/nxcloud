use color_eyre::Result;
use core::fmt;
use core::str::FromStr;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use url::{ParseError, Url};

/// Structure for storing user credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: Username,
    pub password: Password,
    pub server: Server,
}

/// NextCloud Username
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Username(String);

/// NextCloud App Password
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Password(String);

/// NextCloud Server
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Server(Url);

impl Username {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

impl FromStr for Username {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Password {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //f.debug_tuple("Password").field(&"<hidden>").finish()
        f.debug_tuple("Password").field(&self.0).finish()
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<hidden>")
    }
}

impl FromStr for Password {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Server {
    pub fn new(u: Url) -> Result<Self, ParseError> {
        let mut u = u;
        if u.set_scheme("https").is_err() {
            return Err(ParseError::RelativeUrlWithoutBase);
        }
        Ok(Self(u))
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Server {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let u = if s.starts_with("http://") || s.starts_with("https://") {
            Url::parse(s)?
        } else {
            let fqdn = format!("{}{}", "https://", s);
            Url::parse(&fqdn)?
        };
        Self::new(u)
    }
}

impl Credentials {
    /// Returns a Result with Credentials object or ParseError if an invalid url is supplied
    ///
    /// # Arguments
    /// `username` - String slice that represent a NextCloud login username  
    /// `password` - String slice that represents a NextCloud app password  
    /// `server` - String slice that represents a NextCloud server url, http or https can be omitted, https is the default
    ///
    /// # Examples
    /// ```
    /// let creds = Credentials::parse("user", "pass", "www.example.com")
    /// let creds = Credentials::parse("user", "pass", "https://www.example.com")
    /// ```
    pub fn parse(username: &str, password: &str, server: &str) -> Result<Self> {
        let username = Username::new(username.to_string());
        let password = Password::new(password.to_string());
        let server = Server::from_str(server)?;
        Ok(Credentials::new(username, password, server))
    }

    /// Parse the credientials as space sepererated base64 encoded values
    pub fn parse_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::decode(&content)
    }

    /// Create a new Credentials object
    pub fn new(username: Username, password: Password, server: Server) -> Self {
        Self { username, password, server }
    }

    /// Create an encoded string
    pub fn encode(&self) -> String {
        let serialized = serde_json::to_string(&self).unwrap();
        base64::encode(serialized)
    }

    /// Decode a encoded string into a credentials object
    pub fn decode(content: &str) -> Result<Self> {
        let decode = base64::decode(content)?;
        let decoded_str = String::from_utf8_lossy(&decode);
        let deserialized: Self = serde_json::from_str(&decoded_str)?;
        Ok(deserialized)
    }
}
