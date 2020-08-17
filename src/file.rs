use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

const CONFIG: &str = "config.txt";

pub fn read_user(path: &Path) -> Result<(String, String), String> {
    let contents = fs::read_to_string(path);
    let res = match contents {
        Ok(i) => i,
        Err(_) => return Err(String::from("Failed to read file")),
    };
    let v: Vec<&str> = res.split(' ').collect();

    if v.len() != 2 {
        return Err(String::from("Unexpect format"));
    }
    let user = String::from(v[0]);
    let pass = String::from(v[1]);

    return Ok((user, pass));
}

pub fn write_user(user: &str, pass: &str, path: &Path) -> Result<(), io::Error> {
    remove_file(path);

    let contents = format!("{} {}", user, pass);
    let mut file = File::create(&path)?;
    file.write(contents.as_bytes())?;
    return Ok(());
}

fn remove_file(path: &Path) -> bool {
    if path.exists() && path.is_file() {
        fs::remove_file(path).expect("Error: Failed to file");
        return true;
    } else {
        return false;
    }
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_user_no_file() {
        let path = Path::new("test_user_no_file.txt");
        remove_file(path);
        read_user(path).expect_err("File should not exist");
    }

    #[test]
    fn write_user_no_file() {
        let path = Path::new("test_write_user_no_file.txt");
        write_user("user", "pass", path).expect("File should be created");
        assert!(remove_file(path));
    }

    #[test]
    fn write_user_overwrite_file() {
        let path = Path::new("test_write_user_overwrite_file.txt");
        write_user("user", "pass", path).expect("File should be created");
        write_user("user2", "pass2", path).expect("File should be created");
        assert_eq!(
            read_user(path).unwrap(),
            (String::from("user2"), String::from("pass2"))
        );
        assert!(remove_file(path));
    }

    #[test]
    fn write_and_read() {
        let path = Path::new("test_read_and_write.txt");
        remove_file(path);
        write_user("user", "pass", path).expect("File should be created");
        assert_eq!(
            read_user(path).unwrap(),
            (String::from("user"), String::from("pass"))
        );
        assert_ne!(
            read_user(path).unwrap(),
            (String::from("pass"), String::from("user"))
        );
        assert!(remove_file(path));
    }
}
