use std::fs;
use std;
use std::fs::OpenOptions;
use std::path::Path;
use std::process::Command;

pub fn create_dir(dir_name: &str) {
    match fs::create_dir(dir_name) {
        Err(e) => {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                println!("Unable to create dir '{}': {:?}", dir_name, e.kind());
            }
        }
        Ok(_) => {}
    }
}

pub fn touch(filename: &str) {
    let path = Path::new(filename);

    match OpenOptions::new().create(true).write(true).open(path) {
        Ok(_) => {}
        Err(e) => {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                println!("Unable to create file '{}': {:?}", filename, e.kind());
            }
        }
    }
}

pub fn clean_build(project_name: &str) {
    Command::new("rm")
        .arg("-r")
        .arg(format!("{}/build", project_name))
        .output()
        .expect("failed to remove build directory");
}


#[cfg(test)]
mod tests {
    use super::*;

    static TEST_DIR: &str = "test_dir";

    fn setup() {
        create_dir(TEST_DIR);
    }

    fn teardown() {
        Command::new("rm")
            .arg("-r")
            .arg(TEST_DIR)
            .output()
            .expect("failed to remove");
    }

    #[test]
    fn create_file() {
        setup();

        let path = &format!("{}/test_fle", TEST_DIR);
        touch(path);

        let meta = fs::metadata(path).unwrap();
        assert!(meta.is_file());

        teardown()
    }

    #[test]
    fn create_directory() {
        setup();

        let meta = fs::metadata(TEST_DIR).unwrap();
        assert!(meta.is_dir());

        teardown()
    }

    #[test]
    fn delete_build() {
        setup();

        let path = &format!("{}/build", TEST_DIR);
        create_dir(path);

        let meta = fs::metadata(path).unwrap();
        assert!(meta.is_dir());

        clean_build(TEST_DIR);

        let meta_result = fs::metadata(path);

        match meta_result {
            Ok(_) => {
                assert!(false);
            }
            Err(_) => {
                assert!(true);
            }
        }

        teardown()
    }
}
