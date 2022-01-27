use color_eyre::Result;
use path_dedot::ParseDot;
use std::{
    convert::TryFrom,
    ffi::OsStr,
    fmt::{self, Display},
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug, Clone)]
pub struct RemotePathBuf {
    path: PathBuf,
    has_filename: bool,
}

impl RemotePathBuf {
    /// Creates a new empty RemotePathBuf
    pub fn new() -> Self {
        Self { path: PathBuf::new(), has_filename: true }
    }

    /// Removes the prefix from the path /, .., or .,
    fn clean(&mut self) -> Result<()> {
        let path = Path::new("/").join(self.path.clone());

        if path.ends_with(".") || path.ends_with("/") {
            self.has_filename = false;
        }

        // remove the dots
        let dedot_path = path.parse_dot()?;

        // remove double '/' or '\'
        let clean_path: PathBuf = dedot_path.components().collect();

        // strip the starting '/'
        if let Ok(path) = clean_path.strip_prefix("/") {
            self.path = path.to_path_buf();
        } else {
            self.path = clean_path;
        };
        Ok(())
    }

    /// Join a Path to then end, if the Path starts with a / it will start again at the root
    pub fn join(&self, new: &Path) -> Result<RemotePathBuf> {
        let path_buf = self.path.join(new);
        let mut path = Self { path: path_buf, has_filename: self.has_filename };
        path.clean()?;
        Ok(path)
    }

    pub fn to_str(&self) -> Option<&str> {
        self.path.to_str()
    }

    pub fn as_path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn is_file(&self) -> bool {
        self.has_filename
    }

    pub fn set_file_name<S: AsRef<OsStr>>(&mut self, file_name: S) {
        if self.is_file() {
            self.path.pop();
        } else {
            self.has_filename = true;
        }
        self.path.push(file_name.as_ref());
    }
}

impl FromStr for RemotePathBuf {
    type Err = color_eyre::eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path_buf = PathBuf::try_from(s)?;
        let mut path = Self { path: path_buf, has_filename: true };
        path.clean()?;
        Ok(path)
    }
}

impl TryFrom<PathBuf> for RemotePathBuf {
    type Error = color_eyre::eyre::Error;
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let mut path = Self { path: value, has_filename: true };
        path.clean()?;
        Ok(path)
    }
}

impl Display for RemotePathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_dedot() {
        let path = RemotePathBuf::from_str("../../foo/./bar/test.txt");
        assert!(path.is_ok());
        assert_eq!(path.unwrap().to_str().unwrap(), "foo/bar/test.txt");

        let path = RemotePathBuf::from_str("/ab/.");
        assert!(path.is_ok());
        assert_eq!(path.unwrap().to_str().unwrap(), "ab");
    }

    #[test]
    fn clean_deslash() {
        let path = RemotePathBuf::from_str("//////ab");
        assert!(path.is_ok());
        assert_eq!(path.unwrap().to_str().unwrap(), "ab");

        let path = RemotePathBuf::from_str("//////");
        assert!(path.is_ok());
        assert_eq!(path.unwrap().to_str().unwrap(), "");

        let path = RemotePathBuf::from_str(".....///..///test/");
        assert!(path.is_ok());
        assert_eq!(path.unwrap().to_str().unwrap(), "test");

        let path = RemotePathBuf::from_str("/test///////");
        assert!(path.is_ok());
        assert_eq!(path.unwrap().to_str().unwrap(), "test");
    }

    #[test]
    fn clean_join() {
        let path1 = RemotePathBuf::from_str("path");
        assert!(path1.is_ok());

        let path2 = Path::new("path");

        let path = path1.unwrap().join(path2);
        assert!(path.is_ok());
        assert_eq!(path.unwrap().to_str().unwrap(), "path/path");

        // try moving back
        let path1 = RemotePathBuf::from_str("path/path");
        assert!(path1.is_ok());

        let path2 = Path::new("../path");

        let path = path1.unwrap().join(path2);
        assert!(path.is_ok());
        assert_eq!(path.unwrap().to_str().unwrap(), "path/path");

        // not too far
        let path1 = RemotePathBuf::from_str("path/path");
        assert!(path1.is_ok());

        let path2 = Path::new("../../../../path");

        let path = path1.unwrap().join(path2);
        assert!(path.is_ok());
        assert_eq!(path.unwrap().to_str().unwrap(), "path");

        // single dots
        let path1 = RemotePathBuf::from_str("path");
        assert!(path1.is_ok());

        let path2 = Path::new("././././path");

        let path = path1.unwrap().join(path2);
        assert!(path.is_ok());
        assert_eq!(path.unwrap().to_str().unwrap(), "path/path");

        // weird second path
        let path1 = RemotePathBuf::from_str("root");
        assert!(path1.is_ok());

        let path2 = Path::new("///path///");

        let path = path1.unwrap().join(path2);
        assert!(path.is_ok());
        assert_eq!(path.unwrap().to_str().unwrap(), "path");

        // weird second path
        let path1 = RemotePathBuf::from_str("long/path/that/we/are/at");
        assert!(path1.is_ok());

        let path2 = Path::new("/new/path");

        let path = path1.unwrap().join(path2);
        assert!(path.is_ok());
        assert_eq!(path.unwrap().to_str().unwrap(), "new/path");
    }
}
