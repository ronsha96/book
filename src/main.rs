mod commands;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the bookshelf directory
    #[arg(short, long)]
    bookshelf_path: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add an svg icon file to bookshelf
    AddIcon {
        /// Path to the svg icon's file
        path: PathBuf,

        /// The icon's target category
        #[arg(
            short,
            long,
            help = "The icon's category name, formatted in kebab-case. Creates a new directory if the category doesn't exist."
        )]
        category: String,
    },
}

fn main() -> Result<()> {
    let Cli {
        command,
        bookshelf_path,
    } = Cli::parse();

    let bookshelf_path = bookshelf_path.unwrap_or(std::env::current_dir()?);

    is_bookshelf_environment(&bookshelf_path)?;

    match command {
        Commands::AddIcon { path, category } => {
            commands::add_icon(&bookshelf_path, &path, category)
        }
    }
}

fn is_bookshelf_environment(bookshelf_path: &Path) -> Result<()> {
    let path = bookshelf_path.join("package.json");

    let file =
        File::open(path).context("Failed to open package.json in the current working directory")?;

    let package_json: PackageJson = serde_json::from_reader(BufReader::new(file))
        .context("Failed to open package.json in the current working directory")?;

    if package_json.name == "@connecteam/bookshelf" {
        Ok(())
    } else {
        Err(anyhow!(
            "This must be executed inside the bookshelf package, found {} instead",
            package_json.name
        ))
    }
}

#[derive(Deserialize)]
struct PackageJson {
    name: String,
}
