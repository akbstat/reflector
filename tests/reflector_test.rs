use crate::data::RENDER_DATA;
use reflector::{
    acrf::builder::{ACrfBuilder, BuildParam},
    edc::db::DBStruct,
};
use std::path::Path;

#[test]
fn acrf_builder_test() -> anyhow::Result<()> {
    let dbstruct = serde_json::from_slice::<DBStruct>(RENDER_DATA.as_bytes())?;
    let mut builder = ACrfBuilder::new(dbstruct);
    let workspace = Path::new(r"D:\projects\rusty\acrf_outline\.data\ecollect");
    let source = Path::new(r"D:\projects\rusty\acrf_outline\.data\ecollect\acrf_demo.pdf");
    let destination = Path::new(r"D:\projects\rusty\acrf_outline\.data\ecollect\result.pdf");
    let bookmark_bin = Path::new(r"D:\projects\rusty\acrf_outline\.data\ecollect\bookmark.exe");
    builder.build(BuildParam {
        workspace,
        source,
        destination,
        bookmark_bin,
    })?;
    Ok(())
}
