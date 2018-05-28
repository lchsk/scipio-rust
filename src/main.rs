extern crate clap;
extern crate regex;
extern crate chrono;
extern crate pulldown_cmark;
#[macro_use] extern crate slugify;

use pulldown_cmark::{html, Parser};
use slugify::slugify;
use std::process::Command;

use regex::Regex;
use std::io::prelude::*;

use clap::{Arg, App, SubCommand};
use std::fs;
use std::fs::File;
use std::collections::HashMap;
use chrono::prelude::*;

mod filesystem;

#[derive(Debug)]
struct SourceFile {
    source: String,
    title: String,
    stem: String,
    date: DateTime<Utc>,
    body: String,
    entry_type: EntryType,
}

fn open_source_file(source_info: &InternalFile) -> SourceFile {
    let source_path = &source_info.path;
    let stem = &source_info.stem;
    let mut source_f = File::open(source_path).expect("file not found");

    let mut source_contents = String::new();

    source_f.read_to_string(&mut source_contents)
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
                println!("title tag not found in source file {}, skipping", source_path);
            }
        }
    }

    let date: DateTime<Utc>;

    let re = Regex::new(r"created: (?P<date>.+)").unwrap();
    {
        let caps = re.captures(&source_contents);

        match caps {
            Some (dat) => {
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

fn generate_file(
    project_name: &str,
    theme_path: &str,
    source_path: &str,
    output_filename: &str,
    files: &HashMap<String, SourceFile>,
    file_stem: &str,
) {
    let theme_f = File::open(theme_path);
    let mut theme_contents = String::new();

    match theme_f {
        Ok(mut theme_f) => {
            theme_f.read_to_string(&mut theme_contents)
                .expect("something went wrong reading the file");
        }

        Err(_) => {
            println!("Theme file {} not found, skipping", theme_path);
        }
    }

    let source = &files[file_stem];

    let output = theme_contents.replace("{{title}}", &source.title);
    let output = output.replace("{{date}}", &source.date.format("%Y-%m-%d").to_string());
    let mut output = output.replace("{{body}}", &source.body);

    for (page, _) in files {
        let t = &files[page].title;

        let tt = &format!("<a href=\"{}.html\">{}</a>", page, t);

        let st = format!("{{{{@{page}}}}}", page=page);

        output = output.replace(&st, tt);
    }

    let of2 = &format!("{}.html", slugify!(&source.title));
    let mut of: &str = "index.html";

    if output_filename == "" {
        of = of2;
    } else {
        let mut links: Vec<String> = Vec::new();
        let mut posts: Vec<&SourceFile> = Vec::new();

        for (page, page_data) in files {
            let t = &files[page].title;

            let tt = format!("<a href=\"{}.html\">{}</a>", page, t);

            links.push(tt);

            if page_data.entry_type == EntryType::Post {
                posts.push(page_data);
            }
        }

        posts.sort_by(|a, b| b.date.cmp(&a.date));

        let re = Regex::new(r"(?s)\{\{posts-begin\}\}(?P<post>.*)\{\{posts-end\}\}").unwrap();

        let link_tmpl: String;
        {
            let caps = re.captures(&output);

            match caps {
                Some(caps) => {
                    link_tmpl = caps["post"].to_string();
                }
                None => {
                    link_tmpl = "".to_string();
                    println!("Tags posts-begin posts-end not found in {}, skipping", source_path);
                }
            }
        }

        let mut all_links: String = String::new();

        for link in &posts {
            let link_html = format!("<a title=\"{}\" href=\"{}.html\">{}</a>", link.title, link.stem, link.title);
            let link_text = &link_tmpl.replace("{{post_link}}", &link_html).replace("{{date}}", &link.date.format("%Y-%m-%d").to_string());
            all_links.push_str(link_text);
        }

        Command::new("cp")
            .arg("-r")
            .arg(format!("./{}/themes/default/static", project_name))
            .arg(format!("./{}/build", project_name))
            .output()
            .expect("failed to execute process");

        Command::new("cp")
            .arg("-r")
            .arg(format!("./{}/source/data", project_name))
            .arg(format!("./{}/build", project_name))
            .output()
            .expect("failed to execute process");

        output = output.replace("{{posts-begin}}", &all_links);
        output = output.replace("{{posts-end}}", "");
        output = output.replace(&link_tmpl, "");
    }

    let mut file = File::create(format!("{}/{}/{}", project_name, "build", of)).expect("Unable to create file");

    file.write_all(output.as_bytes()).expect("Unable to write into the file");
}

#[derive(Debug)]
struct InternalFile {
    stem: String,
    path: String,
}

fn get_file_stem(path: &std::fs::DirEntry) -> InternalFile {
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

#[derive(Debug)]
#[derive(PartialEq)]
enum EntryType {
    Index,
    Post,
    Page,
}

fn generate(project_name: &str) {
    println!("Generating project '{}'...", project_name);

    let build_dir = format!("{}/{}", project_name, "build");
    filesystem::create_dir(&build_dir);

    let mut files: HashMap<String, SourceFile> = HashMap::new();

    let paths = fs::read_dir(format!("./{}/source/pages", project_name)).unwrap();

    for path in paths {
        match path {
            Ok(path) => {
                let file_info = get_file_stem(&path);
                let source_data = open_source_file(&file_info);

                files.insert(file_info.stem, source_data);
            }
            Err(_) => println!("Invalid file"),
        }
    }

    let paths = fs::read_dir(format!("./{}/source/posts", project_name)).unwrap();

    for path in paths {
        match path {
            Ok(path) => {
                let file_info = get_file_stem(&path);

                let source_data = open_source_file(&file_info);

                files.insert(file_info.stem, source_data);
            }
            Err(_) => println!("Invalid file"),
        }
    }

    let paths = fs::read_dir(format!("./{}/source/", project_name)).unwrap();

    for path in paths {
        match path {
            Ok(path) => {
                let file_info = get_file_stem(&path);

                if file_info.stem == "" {
                    continue;
                }

                let source_data = open_source_file(&file_info);


                files.insert(file_info.stem, source_data);
            }
            Err(_) => println!("Invalid file"),
        }
    }

    for (stem, file) in &files {
        let entry_theme: &str;
        let output_filename: &str;
        let entry_subpath: &str;

        if file.entry_type == EntryType::Page {
            entry_theme = "page.html";
            output_filename = "";
            entry_subpath = "pages/";
        } else if file.entry_type == EntryType::Post {
            entry_theme = "post.html";
            output_filename = "";
            entry_subpath = "posts/";
        } else {
            entry_theme = "index.html";
            output_filename = "index.html";
            entry_subpath = "";
        }

        println!("\t=> Generating '{}'", stem);
        generate_file(project_name, &format!("./{}/themes/default/{}", project_name, entry_theme), &format!("./{}/source/{}{}.md", project_name, entry_subpath, stem), output_filename, &files, &stem);
    }
}

fn create_new_project(project_name: &str) {
    println!("Creating new project '{}'", project_name);

    filesystem::create_dir(project_name);

    let source_dir = format!("{}/{}", project_name, "source");

    filesystem::create_dir(&source_dir);

    let build_dir = format!("{}/{}", project_name, "build");

    filesystem::create_dir(&build_dir);

    let posts_dir = format!("{}/{}", project_name, "source/posts");
    let pages_dir = format!("{}/{}", project_name, "source/pages");

    filesystem::create_dir(&posts_dir);
    filesystem::create_dir(&pages_dir);

    let themes_dir = format!("{}/{}", project_name, "themes");
    filesystem::create_dir(&themes_dir);

    let config_file_dir = format!("{}/{}", project_name, "scipio.config");
    let index_dir = format!("{}/{}", project_name, "source/index.md");

    filesystem::touch(&config_file_dir);
    filesystem::touch(&index_dir);
}

fn main() {
    let matches = App::new("scipio")
        .version("0.1.0")
        .author("lchsk <mjlechowski@gmail.com>")
        .about("simple static website generator")
        .subcommand(
            SubCommand::with_name("create")
                .about("create new project")
                .version("0.1.0")
                .arg(
                    Arg::with_name("project_name")
                        .takes_value(true)
                        .required(true)
                        .help("name of new project"),
                )
        )
        .subcommand(
            SubCommand::with_name("generate")
                .about("generate HTML output")
                .version("0.1.0")
                .arg(
                    Arg::with_name("project_name")
                        .takes_value(true)
                        .required(true)
                        .help("name of the project"),
                )
        )
        .subcommand(
            SubCommand::with_name("clean-build")
                .about("clean build")
                .version("0.1.0")
                .arg(
                    Arg::with_name("project_name")
                        .takes_value(true)
                        .required(true)
                        .help("name of the project"),
                )
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("create") {
        let project_name = matches.value_of("project_name").unwrap_or("scipio_default");
        create_new_project(project_name);
    } else if let Some(matches) = matches.subcommand_matches("generate") {
        let project_name = matches.value_of("project_name").unwrap_or("scipio_default");
        generate(project_name);
    } else if let Some(matches) = matches.subcommand_matches("clean-build") {
        let project_name = matches.value_of("project_name").unwrap_or("scipio_default");
        filesystem::clean_build(project_name);
    }
}
