use anyhow::{Context, Result};
use convert_case::{Case, Casing};
use std::{
    borrow::Cow,
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::Path,
};
use xml::{attribute::Attribute, name::Name};

pub fn add_icon(bookshelf_path: &Path, source_path: &Path, category: String) -> Result<()> {
    let icon_file = File::open(source_path)?;

    let icon_content =
        fix_icon_content(icon_file).context("Failed parsing & modifying icon contents")?;

    let target_dir = bookshelf_path
        .join("src/v2/icons")
        .join(category.to_case(Case::Kebab));

    let icon_file_name = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.to_case(Case::Kebab))
        .context("Icon path doesn't have a file stem")?;

    let mut target_icon_path = target_dir.join(icon_file_name);

    if let Some(ext) = source_path.extension() {
        target_icon_path.set_extension(ext);
    }

    fs::create_dir_all(&target_dir)?;
    fs::write(&target_icon_path, icon_content)?;

    println!("✅ Written the icon to {target_icon_path:?}");

    Ok(())
}

fn fix_icon_content(icon_file: File) -> Result<String> {
    let fill_attr_name = Name::from("fill");

    let mut writer = xml::EmitterConfig::new()
        .perform_indent(true)
        .create_writer(BufWriter::new(Vec::new()));

    let parser = xml::EventReader::new(BufReader::new(icon_file));

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
