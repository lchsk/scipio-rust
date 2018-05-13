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
        .arg(format!("./{}/build", project_name))
        .output()
        .expect("failed to execute process");
}
