use crate::ecrf::ECRF;
use lopdf::Document;
use std::{collections::HashMap, path::Path};

/// store the form and corresponding page information form ecrf
pub struct RaveECRF {
    data: HashMap<String, usize>,
}

impl RaveECRF {
    pub fn new<P: AsRef<Path>>(file: P) -> anyhow::Result<RaveECRF> {
        let doc = Document::load(file)?;
        let bookmarks = doc.get_toc()?;
        let mut data = HashMap::with_capacity(bookmarks.toc.len());
        for form in bookmarks.toc {
            data.insert(form.title, form.page);
        }
        Ok(RaveECRF { data })
    }
}

impl ECRF for RaveECRF {
    fn form_page(&self, form: &str) -> Option<usize> {
        self.data.get(form).map(|page| *page)
    }

    fn list_forms(&self) -> Vec<String> {
        let mut list = self
            .data
            .keys()
            .cloned()
            .into_iter()
            .map(|s| (s.clone(), *self.data.get(&s).unwrap()))
            .collect::<Vec<(String, usize)>>();
        list.sort_by(|a, b| a.1.cmp(&b.1));
        list.into_iter().map(|(s, _)| s).collect()
    }
}

#[test]
fn read_ecrf_test() -> anyhow::Result<()> {
    let ecrf = Path::new(
        r"D:\projects\rusty\acrf_outline\.data\rave\AK112-303_Unique eCRF_V2.0_20230407.pdf",
    );
    let ecrf = RaveECRF::new(ecrf)?;
    assert_eq!(ecrf.form_page("Blood Chemistry".into()), Some(48));
    assert_eq!(ecrf.form_page("Target Lesion".into()), Some(59));
    Ok(())
}
