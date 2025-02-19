use super::xml::{read_rave_config_xml, XmlConfig};
use crate::{
    ecrf::ECRF,
    edc::db::{DBStruct, DBStructReader, Form, FormVisitBinding, Visit},
};
use std::{collections::HashMap, path::Path};

pub struct RaveDBStructReader;

impl RaveDBStructReader {
    pub fn new() -> Self {
        RaveDBStructReader {}
    }
}

impl<P: AsRef<Path>> DBStructReader<P> for RaveDBStructReader {
    fn read(&self, p: P, ecrf: Box<dyn ECRF>) -> anyhow::Result<DBStruct> {
        let XmlConfig {
            forms,
            folders,
            matrixs,
        } = read_rave_config_xml(p)?;
        let mut visit = Vec::with_capacity(forms.len());
        let mut form = Vec::with_capacity(folders.len());
        let mut binding = Vec::with_capacity(forms.len());
        let form_map = build_form_map(&forms);
        let folder_map = build_folders_map(&folders);
        // handle matrix sheet
        let visits = matrixs.first().cloned().unwrap_or_default();
        let visits = visits.get(1..).unwrap_or_default().to_vec();
        for (index, visit_oid) in visits.iter().enumerate() {
            if let Some(visit_oid) = visit_oid {
                if let Some(name) = folder_map.get(visit_oid) {
                    visit.push(Visit {
                        id: index,
                        name: name.to_owned(),
                        order: index as i32,
                    });
                } else if visit_oid.eq("Subject") {
                    visit.push(Visit {
                        id: index,
                        name: visit_oid.to_string(),
                        order: index as i32,
                    });
                }
            }
        }
        let form_visit = matrixs.get(1..).unwrap_or_default();
        for (form_id, row) in form_visit.iter().enumerate() {
            if !row.is_empty() {
                let form_oid = row.first().unwrap();
                if let Some(form_oid) = form_oid {
                    let form_name = form_map.get(form_oid);
                    if let Some(form_name) = form_name {
                        let form_page = ecrf.form_page(form_name).unwrap_or_default();
                        form.push(Form {
                            id: form_id,
                            name: form_name.clone(),
                            page: form_page,
                            order: form_id as i32,
                        });
                    }
                }
                if let Some(row) = row.get(1..) {
                    let mut children = vec![];
                    for (folder_id, folder_oid) in row.iter().enumerate() {
                        if folder_oid.is_some() {
                            children.push(folder_id);
                        }
                    }
                    binding.push(FormVisitBinding {
                        parent: form_id,
                        children,
                    });
                }
            }
        }
        Ok(DBStruct {
            visit,
            form,
            binding,
        })
    }
}

/// build hash map for forms, return HashMap<form oid, form name>
fn build_form_map(sheet: &[Vec<Option<String>>]) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(sheet.len() - 1);
    sheet.iter().enumerate().for_each(|(index, row)| {
        if index > 0 && row.len().gt(&2) {
            let oid = row.get(0).unwrap();
            let name = row.get(2).unwrap();
            if let Some(oid) = oid {
                if let Some(name) = name {
                    map.insert(oid.clone(), name.clone());
                }
            }
        }
    });
    map
}

/// build hash map for folders, return HashMap<folder oid, folder name>
fn build_folders_map(sheet: &[Vec<Option<String>>]) -> HashMap<String, String> {
    // because the position is the same with sheet forms, so just call build_form_map to build the folder map
    build_form_map(sheet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edc::rave::ecrf::RaveECRF;

    #[test]
    fn read_rave_db_test() -> anyhow::Result<()> {
        let ecrf = Path::new(
            r"D:\projects\rusty\acrf_outline\.data\rave\AK112-303_Unique eCRF_V2.0_20230407.pdf",
        );
        let ecrf = RaveECRF::new(ecrf)?;
        let p =
            Path::new(r"D:\projects\rusty\acrf_outline\.data\rave\AK112-303_ALS_V2.0_20230407.xml");
        let reader = RaveDBStructReader::new();
        let result = reader.read(p, Box::new(ecrf))?;
        assert_eq!(result.form.len(), 61);
        assert_eq!(result.visit.len(), 57);
        Ok(())
    }
}
