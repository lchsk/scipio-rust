extern crate markdown;
extern crate clap;
use clap::{Arg, App, SubCommand};
use std::fs;
use std::path::Path;
use std::fs::{OpenOptions};

fn create_dir(dir_name: &str) {
    match fs::create_dir(dir_name) {
        Err(e) => println!("Unable to create '{}': {:?}", dir_name, e.kind()),
        Ok(_) => {},
    }

}


fn touch(filename: &str) {
    let path = Path::new(filename);

    match OpenOptions::new().create(true).write(true).open(path) {
        Ok(_) => {},
        Err(e) => println!("Unable to create file '{}': {:?}", filename, e),
    }
}

fn create_new_project(project_name: &str) {
    println!("Creating new project '{}'", project_name);

    create_dir(project_name);

    let source_dir = format!("{}/{}", project_name, "source");

    create_dir(&source_dir);

    let posts_dir = format!("{}/{}", project_name, "source/posts");
    let pages_dir = format!("{}/{}", project_name, "source/pages");

    create_dir(&posts_dir);
    create_dir(&pages_dir);

    let config_file_dir = format!("{}/{}", project_name, "scipio.config");
    let index_dir = format!("{}/{}", project_name, "source/index.md");

    touch(&config_file_dir);
    touch(&index_dir);
}

fn main() {
    println!("Hello, world!");

    let html : String = markdown::to_html("__I am markdown__ [test](adsd)");

    println!("{}", html);

    let matches = App::new("scipio")
                          .version("0.1.0")
                          .author("lchsk <mjlechowski@gmail.com>")
                          .about("simple static website generator")
                          .subcommand(SubCommand::with_name("create")
                                      .about("create new project")
                                      .version("0.1.0")
                                      .arg(Arg::with_name("project_name")
                                           .takes_value(true)
                                           .required(true)
                                          .help("name of new project")))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("create") {
        create_new_project(matches.value_of("project_name").unwrap_or("scipio_default"));
    }
}
