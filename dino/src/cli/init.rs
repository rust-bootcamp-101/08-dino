use std::{fs, path::Path};

use anyhow::Result;
use askama::Template;
use clap::Parser;
use dialoguer::Input;
use git2::Repository;

use crate::CmdExecutor;

#[derive(Debug, Parser)]
pub struct InitOpts {}

#[derive(Template)]
#[template(path = "config.yml.j2")]
struct ConfigFile {
    name: String,
}

#[derive(Template)]
#[template(path = "main.ts.j2")]
struct MainTsFile {}

#[derive(Template)]
#[template(path = ".gitignore.j2")]
struct GitIgnoreFile {}

impl CmdExecutor for InitOpts {
    async fn execute(self) -> Result<()> {
        let name: String = Input::new().with_prompt("Project name").interact_text()?;

        // if current dir is empty then init project, otherwise create new dir and init project
        let current_dir = Path::new(".");
        if fs::read_dir(current_dir)?.next().is_none() {
            init_project(&name, current_dir)?;
        } else {
            let path = current_dir.join(&name);
            init_project(&name, &path)?;
        }
        Ok(())
    }
}

fn init_project(name: &str, path: &Path) -> Result<()> {
    Repository::init(path)?;
    // init config file
    let config = ConfigFile {
        name: name.to_string(),
    };
    fs::write(path.join("config.yml"), config.render()?)?;
    // init main.ts file
    fs::write(path.join("main.ts"), MainTsFile {}.render()?)?;
    // init .gitignore file
    fs::write(path.join(".gitignore"), GitIgnoreFile {}.render()?)?;
    Ok(())
}
