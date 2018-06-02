extern crate chrono;
extern crate clap;
extern crate regex;
#[macro_use]
extern crate slugify;
extern crate pulldown_cmark;
use slugify::slugify;
use std::process::Command;

use regex::Regex;
use std::io::prelude::*;

use clap::{App, Arg, SubCommand};
use std::collections::HashMap;
use std::fs;
use std::fs::File;

mod files;
mod filesystem;

fn generate_file(
    project_name: &str,
    theme_path: &str,
    source_path: &str,
    output_filename: &str,
    files: &HashMap<String, files::SourceFile>,
    file_stem: &str,
) {
    let theme_f = File::open(theme_path);
    let mut theme_contents = String::new();

    match theme_f {
        Ok(mut theme_f) => {
            theme_f
                .read_to_string(&mut theme_contents)
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

        let st = format!("{{{{@{page}}}}}", page = page);

        output = output.replace(&st, tt);
    }

    let of2 = &format!("{}.html", slugify!(&source.title));
    let mut of: &str = "index.html";

    if output_filename == "" {
        of = of2;
    } else {
        let mut links: Vec<String> = Vec::new();
        let mut posts: Vec<&files::SourceFile> = Vec::new();

        for (page, page_data) in files {
            let t = &files[page].title;

            let tt = format!("<a href=\"{}.html\">{}</a>", page, t);

            links.push(tt);

            if page_data.entry_type == files::EntryType::Post {
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
                    println!(
                        "Tags posts-begin posts-end not found in {}, skipping",
                        source_path
                    );
                }
            }
        }

        let mut all_links: String = String::new();

        for link in &posts {
            let link_html = format!(
                "<a title=\"{}\" href=\"{}.html\">{}</a>",
                link.title, link.stem, link.title
            );
            let post_date = link.date.format("%Y-%m-%d").to_string();
            let link_text = &link_tmpl
                .replace("{{post_link}}", &link_html)
                .replace("{{post_date}}", &post_date);
            all_links.push_str(link_text);
        }

        Command::new("cp")
            .arg("-r")
            .arg(format!("{}/themes/default/static", project_name))
            .arg(format!("{}/build", project_name))
            .output()
            .expect("failed to execute process");

        Command::new("cp")
            .arg("-r")
            .arg(format!("{}/source/data", project_name))
            .arg(format!("{}/build", project_name))
            .output()
            .expect("failed to execute process");

        output = output.replace("{{posts-begin}}", &all_links);
        output = output.replace("{{posts-end}}", "");
        output = output.replace(&link_tmpl, "");
    }

    let mut file = File::create(format!("{}/{}/{}", project_name, "build", of))
        .expect("Unable to create file");

    file.write_all(output.as_bytes())
        .expect("Unable to write into the file");
}

fn generate(project_name: &str) {
    println!("Generating project '{}'...", project_name);

    let build_dir = format!("{}/{}", project_name, "build");
    filesystem::create_dir(&build_dir);

    let mut files: HashMap<String, files::SourceFile> = HashMap::new();

    let paths = fs::read_dir(format!("{}/source/pages", project_name)).unwrap();

    for path in paths {
        match path {
            Ok(path) => {
                let file_info = files::get_file_stem(&path);
                let source_data = files::open_source_file(&file_info);

                files.insert(file_info.stem, source_data);
            }
            Err(_) => println!("Invalid file"),
        }
    }

    let paths = fs::read_dir(format!("{}/source/posts", project_name)).unwrap();

    for path in paths {
        match path {
            Ok(path) => {
                let file_info = files::get_file_stem(&path);

                let source_data = files::open_source_file(&file_info);

                files.insert(file_info.stem, source_data);
            }
            Err(_) => println!("Invalid file"),
        }
    }

    let paths = fs::read_dir(format!("{}/source/", project_name)).unwrap();

    for path in paths {
        match path {
            Ok(path) => {
                let file_info = files::get_file_stem(&path);

                if file_info.stem == "" {
                    continue;
                }

                let source_data = files::open_source_file(&file_info);

                files.insert(file_info.stem, source_data);
            }
            Err(_) => println!("Invalid file"),
        }
    }

    for (stem, file) in &files {
        let entry_theme: &str;
        let output_filename: &str;
        let entry_subpath: &str;

        if file.entry_type == files::EntryType::Page {
            entry_theme = "page.html";
            output_filename = "";
            entry_subpath = "pages/";
        } else if file.entry_type == files::EntryType::Post {
            entry_theme = "post.html";
            output_filename = "";
            entry_subpath = "posts/";
        } else {
            entry_theme = "index.html";
            output_filename = "index.html";
            entry_subpath = "";
        }

        println!("\t=> Generating '{}'", stem);
        generate_file(
            project_name,
            &format!("{}/themes/default/{}", project_name, entry_theme),
            &format!("{}/source/{}{}.md", project_name, entry_subpath, stem),
            output_filename,
            &files,
            &stem,
        );
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
                ),
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
                ),
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
                ),
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
