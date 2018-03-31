extern crate markdown;
extern crate clap;
use clap::{Arg, App, SubCommand};
use std::fs;

fn create_dir(dir_name: &str) {
    match fs::create_dir(dir_name) {
        Err(why) => println!("Unable to create '{}': {:?}", dir_name, why.kind()),
        Ok(_) => {},
    }

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
        let project_name = matches.value_of("project_name").unwrap_or("scipio_default");
        println!("Creating new project '{}'", project_name);

        create_dir(project_name);

        let posts_dir = format!("{}/{}", project_name, "posts");
        let pages_dir = format!("{}/{}", project_name, "pages");

        create_dir(&posts_dir);
        create_dir(&pages_dir);
    }
}
