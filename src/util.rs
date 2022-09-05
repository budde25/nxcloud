use super::RemotePathBuf;
use color_eyre::eyre::bail;
use color_eyre::Result;
use path_dedot::ParseDot;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// Formats the source to be url safe for the pull
pub fn format_source_pull(source: &Path) -> Result<PathBuf> {
    // just to through error if its a directory
    get_source_file_name(source)?;

    Ok(path_remove_prefix(source).parse_dot().unwrap().to_path_buf())
}

/// Formats the destination based of the source, does not need to be cleaned up for url unlike push
/// Ex: source data.txt and dest . then return data.txt
pub fn format_destination_pull(
    source: &Path,
    destination: &Path,
) -> Result<PathBuf> {
    let source_file_name = get_source_file_name(source)?;

    let new_file_path = if path_is_file(destination) {
        destination.to_path_buf()
    } else {
        path_with_file_name(destination, Path::new(&source_file_name))
    };

    Ok(new_file_path)
}

/// Formats the destination based of the source, and removes the '/', '..', or '.' prefixes
/// Ex: source data.txt and dest . then return data.txt
pub fn format_destination_push(
    source: &Path,
    destination: &Path,
) -> Result<PathBuf> {
    let source_file_name = get_source_file_name(source)?;

    let new_file_path = if path_is_file(destination) {
        path_remove_prefix(destination).parse_dot().unwrap().to_path_buf()
    } else {
        let fp = path_with_file_name(destination, Path::new(&source_file_name));
        path_remove_prefix(&fp).parse_dot().unwrap().to_path_buf()
    };

    Ok(new_file_path)
}

/// Gets the file name from the source directory, returns Result of OsString or Error String
fn get_source_file_name(source: &Path) -> Result<OsString> {
    if !path_is_file(source) {
        bail!("Source is a directory");
    };

    if let Some(file_name) = source.file_name() {
        Ok(file_name.to_os_string())
    } else {
        bail!("Source has no file name");
    }
}

/// Checks if a generic path is pointing to a file as opposed to a directory
/// Directory is defined atm as ending with '.','..','/','*', though star is just multiple files, cant support it atm
fn path_is_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    !(path_str.ends_with('.')
        || path_str.ends_with('/')
        || path_str.ends_with('*'))
}

/// Removes the prefix from the path /, .., or .,
fn path_remove_prefix(mut path: &Path) -> PathBuf {
    //TODO cleanup, seems like it could be dont better
    while path.strip_prefix(".").is_ok()
        || path.strip_prefix("/").is_ok()
        || path.strip_prefix("..").is_ok()
    {
        path = if let Ok(new_path) = path.strip_prefix(".") {
            new_path
        } else {
            path
        };

        path = if let Ok(new_path) = path.strip_prefix("..") {
            new_path
        } else {
            path
        };

        path = if let Ok(new_path) = path.strip_prefix("/") {
            new_path
        } else {
            path
        };
    }

    path.components().collect()
}

/// Changes the file_name if the path but unlike the default method correctly handles paths ending with a .
fn path_with_file_name(path: &Path, file_name: &Path) -> PathBuf {
    let parent = if let Some(p) = path.parent() {
        if path_is_file(path) {
            p.join(file_name)
        } else {
            p.join(path.file_name().unwrap_or_else(|| OsStr::new("")))
                .join(file_name)
        }
    } else {
        file_name.to_path_buf()
    };
    parent
}

pub fn join_dedot_path(start: PathBuf, end: PathBuf) -> Result<PathBuf> {
    // Overide dot methods cause they tend to fail
    if end.to_str().is_some() && end.to_str().unwrap() == "." {
        return Ok(start);
    }
    if end.to_str().is_some() && end.to_str().unwrap() == ".." {
        return match end.parent() {
            Some(p) => Ok(p.to_path_buf()),
            None => Ok(PathBuf::from("/")),
        };
    }

    if end.starts_with("/") {
        match end.parse_dot() {
            Ok(p) => {
                Ok(PathBuf::from("/")
                    .join(path_remove_prefix(&p.to_path_buf())))
            }
            Err(_) => Ok(PathBuf::from("/")),
        }
    } else {
        match start.join(end).parse_dot() {
            Ok(p) => {
                Ok(PathBuf::from("/")
                    .join(path_remove_prefix(&p.to_path_buf())))
            }
            Err(_) => Ok(PathBuf::from("/")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Credentials;

    use super::*;

    #[test]
    fn is_file() {
        let path = Path::new("file");
        assert!(path_is_file(path));

        let path = Path::new("file.txt");
        assert!(path_is_file(path));

        let path = Path::new("file/");
        assert!(!path_is_file(path));

        let path = Path::new("file/.");
        assert!(!path_is_file(path));

        let path = Path::new("file/..");
        assert!(!path_is_file(path));

        let path = Path::new("file/*");
        assert!(!path_is_file(path));

        let path = Path::new(".");
        assert!(!path_is_file(path));

        let path = Path::new("/");
        assert!(!path_is_file(path));
    }

    #[test]
    fn remove_prefix() {
        let path = Path::new("file");
        assert_eq!(path_remove_prefix(path).to_str().unwrap(), "file");

        let path = Path::new(".");
        assert_eq!(path_remove_prefix(path).to_str().unwrap(), "");

        let path = Path::new("./file.txt");
        assert_eq!(path_remove_prefix(path).to_str().unwrap(), "file.txt");
    }

    #[test]
    fn with_file_name() {
        let path = Path::new("file");
        let file_name = Path::new("file.txt");
        assert_eq!(
            path_with_file_name(path, file_name).to_str().unwrap(),
            "file.txt"
        );

        let path = Path::new(".");
        let file_name = Path::new("file.txt");
        assert_eq!(
            path_with_file_name(path, file_name).to_str().unwrap(),
            "file.txt"
        );

        let path = Path::new("foo/bar");
        let file_name = Path::new("file.txt");
        assert_eq!(
            path_with_file_name(path, file_name).to_str().unwrap(),
            "foo/file.txt"
        );
    }

    #[test]
    fn format_push_name_both() {
        let source = Path::new("source.txt");
        let destination = Path::new("dest.txt");

        assert_eq!(
            format_destination_push(source, destination).unwrap(),
            destination
        );
    }

    #[test]
    fn format_push_source_is_dir() {
        let source = Path::new(".");
        let destination = Path::new(".");
        format_destination_push(source, destination).unwrap_err();

        let source = Path::new("/ab/.");
        let destination = Path::new("src/files");
        format_destination_push(source, destination).unwrap_err();

        let source = Path::new("/");
        let destination = Path::new("src/files");
        format_destination_push(source, destination).unwrap_err();

        // wildcard not supported (yet)
        let source = Path::new("/*");
        let destination = Path::new("src/files");
        format_destination_push(source, destination).unwrap_err();
    }

    #[test]
    fn format_push_dest_is_dir() {
        let source = Path::new("ab/test.txt");
        let destination = Path::new(".");
        assert_eq!(
            format_destination_push(source, destination)
                .unwrap()
                .to_str()
                .unwrap(),
            "test.txt"
        );

        let source = Path::new("ab/test.txt");
        let destination = Path::new("/file/to/.");
        assert_eq!(
            format_destination_push(source, destination)
                .unwrap()
                .to_str()
                .unwrap(),
            "file/to/test.txt"
        );

        let source = Path::new("/root/test.txt");
        let destination = Path::new("file/to/.");
        assert_eq!(
            format_destination_push(source, destination)
                .unwrap()
                .to_str()
                .unwrap(),
            "file/to/test.txt"
        );

        let source = Path::new("/root/test.txt");
        let destination = Path::new("file/to/");
        assert_eq!(
            format_destination_push(source, destination)
                .unwrap()
                .to_str()
                .unwrap(),
            "file/to/test.txt"
        );

        // TODO maybe handle this case but also its invalid syntax
        let source = Path::new("/root/test.txt");
        let destination = Path::new("file/to/*");
        assert_ne!(
            format_destination_push(source, destination)
                .unwrap()
                .to_str()
                .unwrap(),
            "file/to/test.txt"
        );
    }

    #[test]
    fn format_push_dest_is_file() {
        let source = Path::new("/root/test.txt");
        let destination = Path::new("/file/to/bar.txt");
        assert_eq!(
            format_destination_push(source, destination)
                .unwrap()
                .to_str()
                .unwrap(),
            "file/to/bar.txt"
        );

        let source = Path::new("/root/test.txt");
        let destination = Path::new("./file/to/bar.txt");
        assert_eq!(
            format_destination_push(source, destination)
                .unwrap()
                .to_str()
                .unwrap(),
            "file/to/bar.txt"
        );
    }

    #[test]
    fn format_push_dedot() {
        let source = Path::new("/root/../test.txt");
        let destination = Path::new("/file/to/../bar.txt");
        assert_eq!(
            format_destination_push(source, destination)
                .unwrap()
                .to_str()
                .unwrap(),
            "file/bar.txt"
        );

        let source = Path::new("/root/test.txt");
        let destination = Path::new("../../file/to/bar.txt");
        assert_eq!(
            format_destination_push(source, destination)
                .unwrap()
                .to_str()
                .unwrap(),
            "file/to/bar.txt"
        );
    }

    #[test]
    fn format_dest_pull() {
        let source = Path::new("/ab/test.txt");
        let destination = Path::new("foo/bar/bar.txt");
        assert_eq!(
            format_destination_pull(source, destination)
                .unwrap()
                .to_str()
                .unwrap(),
            "foo/bar/bar.txt"
        );

        let source = Path::new("/ab/test.txt");
        let destination = Path::new("foo/bar/.");
        assert_eq!(
            format_destination_pull(source, destination)
                .unwrap()
                .to_str()
                .unwrap(),
            "foo/bar/test.txt"
        );
    }

    #[test]
    fn format_src_pull() {
        let source = Path::new("/ab/.");
        format_source_pull(source).unwrap_err();

        let source = Path::new(".././..//foo/bar/test.txt");
        assert_eq!(
            format_source_pull(source).unwrap().to_str().unwrap(),
            "foo/bar/test.txt"
        );
    }

    #[test]
    fn default_path_dedot_join() {
        let base = PathBuf::from("/");
        let end = PathBuf::from("//");

        assert_eq!(join_dedot_path(base, end).unwrap().to_str().unwrap(), "/");

        let base = PathBuf::from("/");
        let end = PathBuf::from("//////test/");

        assert_eq!(
            join_dedot_path(base, end).unwrap().to_str().unwrap(),
            "/test"
        );

        let base = PathBuf::from("/");
        let end = PathBuf::from("....///..///test/");

        assert_eq!(
            join_dedot_path(base, end).unwrap().to_str().unwrap(),
            "/test"
        );

        let base = PathBuf::from("/");
        let end = PathBuf::from("/../");

        assert_eq!(join_dedot_path(base, end).unwrap().to_str().unwrap(), "/");
    }

    #[test]
    fn weird_middle_path_dedot_join() {
        let base = PathBuf::from("/");
        let end = PathBuf::from("/../");

        assert_eq!(join_dedot_path(base, end).unwrap().to_str().unwrap(), "/");

        let base = PathBuf::from("/");
        let end = PathBuf::from("foo//bar/");

        assert_eq!(
            join_dedot_path(base, end).unwrap().to_str().unwrap(),
            "/foo/bar"
        );
    }

    #[test]
    fn url_no_base() {
        let creds =
            Credentials::parse("username", "password", "www.example.com");
        assert!(creds.is_ok());
    }
}
