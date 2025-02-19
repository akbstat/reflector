use crate::{
    ecrf::ECRF,
    edc::db::{DBStruct, DBStructReader, Form, FormVisitBinding, Visit},
};
use calamine::{open_workbook, DataType, Reader, Xlsx};
use std::path::Path;

const TARGET_SHEET: &str = "EventWorkflow";

pub struct EcollectDBStructReader;

impl EcollectDBStructReader {
    pub fn new() -> Self {
        EcollectDBStructReader {}
    }
}

impl<P: AsRef<Path>> DBStructReader<P> for EcollectDBStructReader {
    fn read(&self, p: P, ecrf: Box<dyn ECRF>) -> anyhow::Result<DBStruct> {
        let mut workbook: Xlsx<_> = open_workbook(p)?;
        let sheet = workbook.worksheet_range(TARGET_SHEET)?;
        let (row, column) = sheet.get_size();
        let mut visit = Vec::with_capacity(column - 1);
        let mut form = Vec::with_capacity(row - 1);
        let mut binding = Vec::with_capacity(row - 1);
        for (index, row) in sheet.rows().into_iter().enumerate() {
            if index.eq(&0) {
                // get visit
                for col in 1..column {
                    if let Some(v) = row.get(col).unwrap().as_string() {
                        let v = v.trim();
                        if v.is_empty() {
                            break;
                        }
                        visit.push(Visit {
                            id: col - 1,
                            name: v.into(),
                            order: col as i32 - 1,
                        });
                    }
                }
            } else {
                // get form
                let id = index - 1;
                let mut b = FormVisitBinding {
                    parent: id,
                    children: vec![],
                };
                if let Some(v) = row.get(0).unwrap().as_string() {
                    let v = v.trim();
                    if v.is_empty() {
                        break;
                    }
                    form.push(Form {
                        id,
                        name: v.into(),
                        page: ecrf.form_page(v).unwrap_or_default(),
                        order: id as i32,
                    });
                }
                for col in 1..column {
                    if let Some(cell) = row.get(col).unwrap().as_string() {
                        if !cell.trim().is_empty() {
                            b.children.push(col - 1);
                        }
                    }
                }
                binding.push(b);
            }
        }
        Ok(DBStruct {
            visit,
            form,
            binding,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edc::ecollect::ecrf::ECollectECRF;

    #[test]
    fn read_ecollect_db_test() -> anyhow::Result<()> {
        let ecrf = Path::new(
            r"D:\projects\rusty\acrf_outline\.data\ecollect\AK120-301_Unique eCRF_V2.0_20240530.pdf",
        );
        let ecrf = ECollectECRF::new(ecrf)?;
        let p = Path::new(
            r"D:\projects\rusty\acrf_outline\.data\ecollect\database_export_AK120-301_20240606_0000.xlsx",
        );
        let reader = EcollectDBStructReader::new();
        let result = reader.read(p, Box::new(ecrf))?;
        assert_eq!(result.form.len(), 50);
        assert_eq!(result.visit.len(), 32);
        Ok(())
    }
}
