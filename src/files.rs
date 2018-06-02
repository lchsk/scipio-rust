extern crate pulldown_cmark;
use chrono::prelude::*;
use pulldown_cmark::{html, Parser};
use regex::Regex;
use std;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
pub struct InternalFile {
    pub stem: String,
    pub path: String,
}

#[derive(Debug)]
pub struct SourceFile {
    pub source: String,
    pub title: String,
    pub stem: String,
    pub date: DateTime<Utc>,
    pub body: String,
    pub entry_type: EntryType,
}

#[derive(Debug, PartialEq)]
pub enum EntryType {
    Index,
    Post,
    Page,
}

pub fn get_file_stem(path: &std::fs::DirEntry) -> InternalFile {
    let path_buf = path.path();

    if !path_buf.is_file() {
        return InternalFile {
            stem: "".to_string(),
            path: "".to_string(),
        };
    }

    let path = path_buf.as_path();

    match path.file_stem() {
        Some(stem) => match stem.to_str() {
            Some(stem) => {
                return InternalFile {
                    stem: stem.to_string(),
                    path: path.to_str().unwrap().to_string(),
                }
            }

            None => {
                println!("Invalid file stem found");
            }
        },
        None => {}
    }

    return InternalFile {
        stem: "".to_string(),
        path: "".to_string(),
    };
}

pub fn open_source_file(source_info: &InternalFile) -> SourceFile {
    let source_path = &source_info.path;
    let stem = &source_info.stem;
    let mut source_f = File::open(source_path).expect("file not found");

    let mut source_contents = String::new();

    source_f
        .read_to_string(&mut source_contents)
        .expect("something went wrong reading the file");

    let title: String;

    let re = Regex::new(r"title: (?P<title>.+)").unwrap();
    {
        let caps = re.captures(&source_contents);

        match caps {
            Some(caps) => {
                title = caps["title"].to_string();
            }
            None => {
                title = "".to_string();
                println!(
                    "title tag not found in source file {}, skipping",
                    source_path
                );
            }
        }
    }

    let date: DateTime<Utc>;

    let re = Regex::new(r"created: (?P<date>.+)").unwrap();
    {
        let caps = re.captures(&source_contents);

        match caps {
            Some(dat) => {
                let d = dat["date"].to_string().parse::<DateTime<Utc>>();

                match d {
                    Ok(valid_date) => {
                        date = valid_date;
                    }
                    Err(_) => {
                        date = Utc::now();
                    }
                }
            }
            None => {
                date = Utc::now();
            }
        }
    }

    let mut body: String;
    {
        let result: Vec<_> = source_contents.lines().collect();
        let mut body_v: Vec<&str> = Vec::new();

        let mut i = 0;

        for line in result {
            if line.starts_with("---") {
                i += 1;
                continue;
            }

            if i >= 2 {
                body_v.push(line);
            }
        }

        let body_text = body_v.join("\n");

        let parser = Parser::new(body_text.trim());
        body = String::new();
        html::push_html(&mut body, parser);
    }

    let entry_type: EntryType;

    if stem == "index" {
        entry_type = EntryType::Index;
    } else if source_path.contains("/posts/") {
        entry_type = EntryType::Post;
    } else {
        entry_type = EntryType::Page;
    }

    SourceFile {
        source: source_contents,
        title: title,
        stem: stem.to_string(),
        date: date,
        body: body,
        entry_type: entry_type,
    }
}
