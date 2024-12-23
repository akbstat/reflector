# Reflector

> A libary for generating TOC and bookmark for aCRF

# Features
- Read EDC building file and eCRF to get the form page information and visit binding relationship
- Generate aCRF with bookmark and TOC

# How to use
> note: binary file for adding bookmark is needed 

```rust
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
```

# TODO
- Support other EDC(only support EDC building file for ecollect now)
- Configuration persistence