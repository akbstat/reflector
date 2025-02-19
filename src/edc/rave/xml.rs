use quick_xml::{
    events::{BytesStart, Event},
    Reader,
};
use std::{fs::File, io::BufReader, path::Path};

const WORKSHEET: &[u8] = b"Worksheet";
const TABLE: &[u8] = b"Table";
const ROW: &[u8] = b"Row";
const CELL: &[u8] = b"Cell";

#[derive(Debug, Default)]
pub struct XmlConfig {
    pub forms: Vec<Vec<Option<String>>>,
    pub folders: Vec<Vec<Option<String>>>,
    pub matrixs: Vec<Vec<Option<String>>>,
}

pub fn read_rave_config_xml<P: AsRef<Path>>(filepath: P) -> anyhow::Result<XmlConfig> {
    let file = File::open(filepath)?;
    let file = BufReader::new(file);
    let mut result = XmlConfig::default();
    let mut reader = Reader::from_reader(file);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(event) => {
                if WORKSHEET.eq(event.name().as_ref()) {
                    // filter target sheets
                    let attrs = read_tag_attributes(&event);
                    match attrs.first().unwrap().as_str() {
                        s if is_target_sheet(s) => {
                            let sheet = read_worksheet_table(&mut reader)?;
                            match s {
                                "Forms" => result.forms = sheet,
                                "Folders" => result.folders = sheet,
                                _ => result.matrixs = sheet,
                            }
                        }
                        _ => continue,
                    }
                }
                buf.clear();
            }
            Event::Eof => break,
            _ => continue,
        }
    }
    Ok(result)
}

fn read_worksheet_table(
    reader: &mut Reader<BufReader<File>>,
) -> anyhow::Result<Vec<Vec<Option<String>>>> {
    let mut sheet: Vec<Vec<Option<String>>> = vec![];
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(event) => {
                if ROW.eq(event.name().as_ref()) {
                    let row = read_row(reader)?;
                    if !row.is_empty() {
                        sheet.push(row);
                    }
                }
                buf.clear();
            }
            Event::End(event) => {
                if TABLE.eq(event.name().as_ref()) {
                    break;
                }
            }
            _ => continue,
        }
    }
    Ok(sheet)
}

fn read_row(reader: &mut Reader<BufReader<File>>) -> anyhow::Result<Vec<Option<String>>> {
    let mut row = vec![];
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(event) => {
                // if cell tag contains ss:Index attribution, means there are empty cells between current and last cell, append empty cells into rows as None
                if CELL.eq(event.name().as_ref()) {
                    if let Some(index) = read_index(&event) {
                        let mut gap = index - 1 - row.len();
                        while gap > 0 {
                            row.push(None);
                            gap -= 1;
                        }
                    }
                }
                row.push(read_cell_data(reader)?);
                buf.clear();
            }
            Event::End(event) => {
                if ROW.eq(event.name().as_ref()) {
                    break;
                }
            }
            _ => continue,
        }
    }
    Ok(row)
}

fn read_cell_data(reader: &mut Reader<BufReader<File>>) -> anyhow::Result<Option<String>> {
    let mut buf = Vec::new();
    let mut cell = None;
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Text(content) => {
                if let None = cell {
                    cell = Some(content.unescape().unwrap().into_owned());
                }
                break;
            }
            _ => continue,
        };
    }
    Ok(cell)
}

fn read_tag_attributes(event: &BytesStart) -> Vec<String> {
    event
        .attributes()
        .map(|a| {
            let v = a.unwrap().value;
            let s = std::str::from_utf8(&v).unwrap();
            s.to_string()
        })
        .collect::<Vec<_>>()
}

fn read_index(event: &BytesStart) -> Option<usize> {
    let attrs = read_tag_attributes(event);
    match attrs.last() {
        Some(index) => match index.parse::<usize>() {
            Ok(index) => Some(index),
            Err(_) => None,
        },
        None => None,
    }
}

fn is_target_sheet(sheet: &str) -> bool {
    sheet.eq("Forms")
        || sheet.eq("Folders")
        || (sheet.starts_with("Matrix") && sheet.ends_with("MASTER"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn read_config_test() -> anyhow::Result<()> {
        let config = Path::new(r"D:\projects\rusty\acrf_outline\.data\rave\test.xml");
        let xml = read_rave_config_xml(config).unwrap();
        assert_eq!(xml.forms.len(), 62);
        Ok(())
    }

    #[test]
    fn test_read_row() {
        let file = File::open(Path::new(
            r"D:\projects\rusty\acrf_outline\.data\rave\row.xml",
        ))
        .unwrap();
        let file = BufReader::new(file);
        let mut reader = Reader::from_reader(file);
        reader.config_mut().trim_text(true);
        let row = read_row(&mut reader).unwrap();
        assert_eq!(row.len(), 17);
    }

    #[test]
    fn test_read_sheet_table() {
        let file = File::open(Path::new(
            r"D:\projects\rusty\acrf_outline\.data\rave\sheet.xml",
        ))
        .unwrap();
        let file = BufReader::new(file);
        let mut reader = Reader::from_reader(file);
        reader.config_mut().trim_text(true);
        let sheet = read_worksheet_table(&mut reader).unwrap();
        assert_eq!(sheet.len(), 62);
    }
}
