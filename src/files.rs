use std;

#[derive(Debug)]
pub struct InternalFile {
    pub stem: String,
    pub path: String,
}

pub fn get_file_stem(path: &std::fs::DirEntry) -> InternalFile {
    let path_buf = path.path();

    if !path_buf.is_file() {
        return InternalFile {
            stem: "".to_string(),
            path: "".to_string(),
        }
    }

    let path = path_buf.as_path();

    match path.file_stem() {
        Some(stem) => {
            match stem.to_str() {
                Some(stem) => {
                    return InternalFile {
                        stem: stem.to_string(),
                        path: path.to_str().unwrap().to_string(),
                    }
                }

                None => {
                    println!("Invalid file stem found");
                }
            }
        }
        None => {
        }
    }

    return InternalFile {
        stem: "".to_string(),
        path: "".to_string(),
    }
}
