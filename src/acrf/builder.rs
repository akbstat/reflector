use super::{combiner::merge_pdf, toc::render::Level};
use crate::{
    acrf::toc::render::{Render, RenderData, RenderParam},
    edc::db::{DBStruct, Form, FormVisitBinding, Visit},
};
use lopdf::Document;
use std::{cell::Cell, collections::HashMap, path::Path};

pub struct ACrfBuilder {
    pub(crate) visit_map: HashMap<usize, Visit>,
    pub(crate) form_map: HashMap<usize, Form>,
    pub(crate) visit_form_binding: HashMap<usize, Vec<usize>>,
    pub(crate) form_visit_binding: HashMap<usize, Vec<usize>>,
    pub(crate) toc_pages: Cell<usize>,
    pub(crate) visit_render_data: Option<RenderData>,
    pub(crate) form_render_data: Option<RenderData>,
}

impl ACrfBuilder {
    pub fn new(db: DBStruct) -> Self {
        let visit_map = db.visit.iter().map(|v: &Visit| (v.id, v.clone())).collect();
        let form_map = db.form.iter().map(|f| (f.id, f.clone())).collect();
        let form_visit_binding = db
            .binding
            .iter()
            .map(|f| (f.parent, f.children.clone()))
            .collect();
        let mut visit_form_binding: HashMap<usize, Vec<usize>> =
            HashMap::with_capacity(db.visit.len());
        db.binding
            .iter()
            .for_each(|FormVisitBinding { parent, children }| {
                children.iter().for_each(|v| {
                    if let Some(children) = visit_form_binding.get_mut(v) {
                        children.push(*parent);
                    } else {
                        visit_form_binding.insert(*v, vec![*parent]);
                    }
                });
            });
        ACrfBuilder {
            visit_map,
            form_map,
            visit_form_binding,
            form_visit_binding,
            toc_pages: Cell::new(0),
            visit_render_data: None,
            form_render_data: None,
        }
    }

    pub fn build<P: AsRef<Path>>(&mut self, param: BuildParam<P>) -> anyhow::Result<()> {
        let BuildParam {
            source,
            destination,
            workspace,
            bookmark_bin,
        } = param;
        self.visit_render_data = Some(self.build_visit_render_data());
        self.form_render_data = Some(self.build_form_render_data());

        // build toc
        let toc = workspace.as_ref().join("toc.pdf");
        self.build_toc(&toc)?;
        self.update_toc_pages(&toc)?;
        // merge toc to acrf
        merge_pdf(&vec![&toc, source.as_ref()], destination.as_ref())?;
        // link toc
        self.link_toc(destination.as_ref())?;
        // link bookmarks
        self.link_bookmark(LinkBookmarkParam {
            target: destination.as_ref(),
            acrf_outline_bin: bookmark_bin.as_ref(),
            workspace: workspace.as_ref(),
        })?;
        Ok(())
    }

    fn build_toc<P: AsRef<Path>>(&self, destination: P) -> anyhow::Result<()> {
        if self.visit_render_data.is_some() || self.form_render_data.is_some() {
            let render = Render::new()?;
            render.write(RenderParam {
                visit: self.visit_render_data.clone().unwrap(),
                form: self.form_render_data.clone().unwrap(),
                destination,
            })?;
        }
        Ok(())
    }

    fn update_toc_pages<P: AsRef<Path>>(&self, toc: P) -> anyhow::Result<()> {
        let toc = Document::load(toc)?;
        self.toc_pages.set(toc.get_pages().len());
        Ok(())
    }

    fn build_visit_render_data(&self) -> RenderData {
        let mut data = RenderData::default();
        data.name = "Visit".into();
        let mut data_children = Vec::with_capacity(self.visit_map.len());
        let mut visits = self
            .visit_map
            .values()
            .map(|v| v.clone())
            .collect::<Vec<_>>();
        visits.sort_by(|x, y| x.order.cmp(&y.order));
        for visit in visits {
            let mut visit_data = RenderData::default();
            visit_data.name = visit.name.clone();
            visit_data.kind = Level::LEVEL2;
            if let Some(forms) = self.visit_form_binding.get(&visit.id) {
                let mut form_list = Vec::with_capacity(forms.len());
                for form_id in forms {
                    if let Some(form) = self.form_map.get(form_id) {
                        form_list.push(form.clone());
                    }
                }
                form_list.sort_by(|x, y| x.order.cmp(&y.order));
                let visit_data_children = form_list
                    .into_iter()
                    .map(|f| {
                        let mut form_data = RenderData::default();
                        form_data.id = Some(f.id);
                        form_data.name = f.name.clone();
                        form_data.kind = Level::LEVEL3;
                        form_data.page = Some(f.page);
                        form_data
                    })
                    .collect::<Vec<_>>();
                visit_data.children = Some(visit_data_children);
                data_children.push(visit_data);
            }
        }
        data.children = Some(data_children);
        data
    }

    fn build_form_render_data(&self) -> RenderData {
        let mut data = RenderData::default();
        data.name = "Forms".into();
        let mut data_children = Vec::with_capacity(self.form_map.len());
        let mut forms = self
            .form_map
            .values()
            .map(|f| f.clone())
            .collect::<Vec<_>>();
        forms.sort_by(|x, y| x.order.cmp(&y.order));
        for form in forms {
            let mut form_data = RenderData::default();
            form_data.kind = Level::LEVEL2;
            form_data.name = form.name.clone();
            if let Some(visits) = self.form_visit_binding.get(&form.id) {
                let mut visit_list = Vec::with_capacity(visits.len());
                for visit_id in visits {
                    if let Some(visit) = self.visit_map.get(visit_id) {
                        visit_list.push(visit.clone());
                    }
                }
                visit_list.sort_by(|x, y| x.order.cmp(&y.order));
                let form_data_children = visit_list
                    .into_iter()
                    .map(|v| {
                        let mut visit_data = RenderData::default();
                        visit_data.id = Some(form.id);
                        visit_data.name = v.name.clone();
                        visit_data.kind = Level::LEVEL3;
                        visit_data.page = Some(form.page);
                        visit_data
                    })
                    .collect::<Vec<_>>();
                form_data.children = Some(form_data_children);
                data_children.push(form_data);
            }
        }
        data.children = Some(data_children);
        data
    }
}

pub struct BuildParam<P: AsRef<Path>> {
    pub source: P,
    pub destination: P,
    pub workspace: P,
    pub bookmark_bin: P,
}

pub struct LinkBookmarkParam<P: AsRef<Path>> {
    pub(crate) target: P,
    pub(crate) acrf_outline_bin: P,
    pub(crate) workspace: P,
}
