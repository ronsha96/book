use anyhow::{anyhow, Context, Result};
use clap::Parser;
use convert_case::{Case, Casing};
use serde::Deserialize;
use std::{
    borrow::Cow,
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};
use xml::{attribute::Attribute, name::Name};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the icon's svg file
    path: PathBuf,

    /// The icon's target category
    category: String,

    /// Path to the bookshelf directory
    #[arg(short, long)]
    bookshelf_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let Cli {
        path: source_path,
        category,
        bookshelf_path,
    } = Cli::parse();

    let bookshelf_path = bookshelf_path.unwrap_or(PathBuf::new());

    is_bookshelf_environment(&bookshelf_path)?;

    let svg_file = File::open(&source_path)?;

    let icon_content =
        fix_icon_content(svg_file).context("Failed parsing & modifying icon contents")?;

    let target_dir = bookshelf_path
        .join("src/v2/icons")
        .join(category.to_case(Case::Kebab));

    let icon_file_name = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.to_case(Case::Kebab))
        .context("Icon path doesn't have a file stem")?;

    let mut target_icon_path = target_dir.join(&icon_file_name);

    if let Some(ext) = source_path.extension() {
        target_icon_path.set_extension(ext);
    }

    fs::create_dir_all(&target_dir)?;
    fs::write(&target_icon_path, &icon_content)?;

    println!("✅ Written the icon to {target_icon_path:?}");

    Ok(())
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
