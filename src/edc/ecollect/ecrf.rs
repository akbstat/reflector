use lopdf::Document;
use std::{collections::HashMap, path::Path};

use crate::ecrf::ECRF;

/// store the form and corresponding page information form ecrf
pub struct ECollectECRF {
    data: HashMap<String, usize>,
}

impl ECollectECRF {
    pub fn new<P: AsRef<Path>>(file: P) -> anyhow::Result<Self> {
        let doc = Document::load(file)?;
        let bookmarks = doc.get_toc()?;
        let mut data = HashMap::with_capacity(bookmarks.toc.len());
        for form in bookmarks.toc {
            data.insert(form.title, form.page);
        }
        Ok(ECollectECRF { data })
    }
}

impl ECRF for ECollectECRF {
    fn form_page(&self, form: &str) -> Option<usize> {
        self.data.get(form).map(|page| *page)
    }
}

#[test]
fn read_ecrf_test() -> anyhow::Result<()> {
    let ecrf = Path::new(
        r"D:\projects\rusty\acrf_outline\.data\ecollect\AK120-301_Unique eCRF_V2.0_20240530.pdf",
    );
    let ecrf = ECollectECRF::new(ecrf)?;
    assert_eq!(ecrf.form_page("访视日期".into()), Some(18));
    assert_eq!(ecrf.form_page("死亡".into()), Some(83));
    Ok(())
}
