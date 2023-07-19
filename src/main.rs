use anyhow::{anyhow, Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::{
    borrow::Cow,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};
use xml::{attribute::Attribute, name::Name};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the icon's svg file
    path: Box<Path>,

    /// The icon's target category
    category: String,

    /// Path to the bookshelf directory
    #[arg(short, long)]
    bookshelf_path: Option<Box<Path>>,
}

fn main() -> Result<()> {
    let Cli {
        path: icon_path,
        category,
        bookshelf_path,
    } = Cli::parse();

    is_bookshelf_environment(bookshelf_path)?;

    // TODO: read svg file
    let svg_file = File::open(&icon_path)?;

    // TODO: change fill in the read svg to currentColor
    let icon_content = fix_icon_content(svg_file)?;

    println!("{icon_content:?}");

    // TODO: write the svg to `src/v2/icons/{category}/{kebab_case(file_name(svg))}`
    // TODO: write an entire directory of svg files:
    // TODO:    1. read all direct children of dir
    // TODO:    2. category = kebab_case(dir_name)
    // TODO:    3. run fill transform on all files
    // TODO:    4. write all svg files to `src/v2/icons/{category}/{...}`

    Ok(())
}

fn is_bookshelf_environment(bookshelf_path: Option<Box<Path>>) -> Result<()> {
    let path = get_bookshelf_path(bookshelf_path, "package.json");

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

fn get_bookshelf_path(bookshelf_path: Option<Box<Path>>, joinee: impl AsRef<Path>) -> PathBuf {
    if let Some(p) = bookshelf_path {
        p.to_path_buf().join(joinee)
    } else {
        PathBuf::new().join(joinee)
    }
}

fn fix_icon_content(svg_file: File) -> Result<String> {
    let fill_attr_name = Name::from("fill");

    let mut writer = xml::EmitterConfig::new()
        .perform_indent(true)
        .create_writer(BufWriter::new(Vec::new()));

    let parser = xml::EventReader::new(BufReader::new(svg_file));

    for ev in parser {
        match ev {
            Ok(xml::reader::XmlEvent::StartElement {
                name,
                attributes,
                namespace,
            }) => {
                let name = name.borrow();

                let attributes = attributes
                    .iter()
                    .map(|attr| {
                        if attr.name.borrow() == fill_attr_name {
                            Attribute::new(attr.name.borrow(), "currentColor")
                        } else {
                            attr.borrow()
                        }
                    })
                    .collect();

                let namespace = Cow::Borrowed(&namespace);

                writer.write(xml::writer::XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace,
                })?;
            }
            Ok(ev) => {
                if let Some(writer_ev) = ev.as_writer_event() {
                    writer.write(writer_ev)?;
                }
            }
            Err(err) => return Err(err).context("Failed parsing the icon file"),
        };
    }

    Ok(String::from_utf8(writer.into_inner().into_inner()?)?)
}
