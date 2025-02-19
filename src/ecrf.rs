use crate::edc::{db::DBKind, ecollect::ecrf::ECollectECRF, rave::ecrf::RaveECRF};
use std::path::Path;

pub trait ECRF {
    fn form_page(&self, form: &str) -> Option<usize>;
    fn list_forms(&self) -> Vec<String>;
}

pub fn ecrf_reader<P: AsRef<Path>>(kind: &DBKind, file: P) -> anyhow::Result<Box<dyn ECRF>> {
    Ok(match kind {
        DBKind::ECollect => Box::new(ECollectECRF::new(file)?),
        DBKind::Rave => Box::new(RaveECRF::new(file)?),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ecrf_reader() -> anyhow::Result<()> {
        let reader = ecrf_reader(
            &DBKind::Rave,
            Path::new(
                r"D:\projects\rusty\acrf_outline\.data\rave\AK112-303_Unique eCRF_V2.0_20230407.pdf",
            ),
        )?;
        let forms = reader.list_forms();
        assert_eq!(forms.len(), 62);

        let reader = ecrf_reader(
            &DBKind::ECollect,
            Path::new(
                r"D:\projects\rusty\acrf_outline\.data\ecollect\AK120-301_Unique eCRF_V2.0_20240530.pdf",
            ),
        )?;
        let forms = reader.list_forms();
        assert_eq!(forms.len(), 52);
        Ok(())
    }
}
