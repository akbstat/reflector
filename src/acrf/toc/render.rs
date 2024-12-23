use super::template::TEMPLATE;
use headless_chrome::{types::PrintToPdfOptions, Browser};
use serde::Serialize;
use std::{
    fs,
    ops::{Add, Sub},
    path::Path,
};
use tera::{Context, Tera};

const TOC_TEMPLATE: &str = "toc";

pub struct Render {
    template: Tera,
}

impl Render {
    pub fn new() -> anyhow::Result<Self> {
        let mut template = Tera::default();
        template.add_raw_template(TOC_TEMPLATE, TEMPLATE)?;
        Ok(Render { template })
    }
    pub fn write<P: AsRef<Path>>(&self, param: RenderParam<P>) -> anyhow::Result<()> {
        let toc = vec![param.visit, param.form];
        let mut context = Context::new();
        context.insert("content", &serde_json::to_string(&toc)?);
        let html_file_name = param
            .destination
            .as_ref()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .replace(".pdf", ".html");
        let html = param
            .destination
            .as_ref()
            .parent()
            .unwrap()
            .join(html_file_name);
        fs::write(
            html.as_path(),
            self.template.render(TOC_TEMPLATE, &context)?,
        )?;
        html_to_pdf(html.as_path(), param.destination.as_ref())?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct RenderData {
    pub(crate) id: Option<usize>,
    pub(crate) kind: Level,
    pub(crate) name: String,
    pub(crate) page: Option<usize>,
    pub(crate) children: Option<Vec<RenderData>>,
}

#[derive(Debug, Serialize, Default, Clone)]
pub enum Level {
    #[default]
    LEVEL1,
    LEVEL2,
    LEVEL3,
}

pub(crate) struct RenderParam<P: AsRef<Path>> {
    pub(crate) visit: RenderData,
    pub(crate) form: RenderData,
    pub(crate) destination: P,
}

impl RenderData {
    pub(crate) fn update_pages(&mut self, base: usize) {
        if let Some(page) = self.page {
            self.page = Some(base.add(page).sub(1));
        } else if let Some(children) = self.children.as_mut() {
            for child in children.iter_mut() {
                child.update_pages(base);
            }
        }
    }
}

pub fn html_to_pdf<P: AsRef<Path>>(source: P, destination: P) -> anyhow::Result<()> {
    let url = format!("file:///{}", source.as_ref().to_string_lossy().to_string());
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;
    let pdf_options: Option<PrintToPdfOptions> = Some(PrintToPdfOptions {
        landscape: None,
        display_header_footer: None,
        print_background: None,
        scale: None,
        paper_width: None,
        paper_height: None,
        margin_top: None,
        margin_bottom: None,
        margin_left: None,
        margin_right: None,
        page_ranges: None,
        ignore_invalid_page_ranges: None,
        header_template: None,
        footer_template: None,
        prefer_css_page_size: Some(true),
        transfer_mode: None,
    });
    let pdf = tab
        .navigate_to(&url)?
        .wait_until_navigated()?
        .print_to_pdf(pdf_options)?;
    fs::write(destination, pdf)?;
    Ok(())
}
