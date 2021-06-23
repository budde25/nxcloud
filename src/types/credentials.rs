use core::fmt;
use core::str::FromStr;
use url::{ParseError, Url};

/// Structure for storing user credentials
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: Username,
    pub password: Password,
    pub server: Server,
}

/// NextCloud Username
#[derive(Debug, Clone, PartialEq)]
pub struct Username(String);

/// NextCloud App Password
#[derive(Clone, PartialEq)]
pub struct Password(String);

/// NextCloud Server
#[derive(Debug, Clone, PartialEq)]
pub struct Server(Url);

impl Username {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Username {
    type Err = Box<dyn std::error::Error>;

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

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Password").field(self).finish()
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Password {
    type Err = Box<dyn std::error::Error>;

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
    pub fn parse(
        username: &str,
        password: &str,
        server: &str,
    ) -> anyhow::Result<Self> {
        let username = Username::new(username.to_string());
        let password = Password::new(password.to_string());
        let server = Server::from_str(server)?;
        Ok(Credentials::new(username, password, server))
    }

    pub fn new(username: Username, password: Password, server: Server) -> Self {
        Self { username, password, server }
    }
}
