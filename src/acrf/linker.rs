use super::builder::{ACrfBuilder, LinkBookmarkParam};
use anyhow::anyhow;
use lopdf::{dictionary, Document, Object, ObjectId};
use std::{fs, ops::Add, os::windows::process::CommandExt, path::Path, process::Command};

impl ACrfBuilder {
    pub fn link_toc<P: AsRef<Path>>(&self, target: P) -> anyhow::Result<()> {
        let mut document = Document::load(target.as_ref())?;
        let obj_ids = document
            .objects
            .iter()
            .map(|(id, _)| id.clone())
            .collect::<Vec<ObjectId>>();
        for id in obj_ids {
            let obj = document.get_object_mut(id)?;
            if let Ok(obj) = obj.as_dict_mut() {
                if obj.type_is(b"Annot") {
                    if let Ok(dest) = obj.get(b"Dest") {
                        if let Ok(dest) = dest.as_name_str() {
                            let id = dest.to_string().parse::<usize>()?;
                            if let Some(f) = self.form_map.get(&id) {
                                obj.set(
                                    b"Dest",
                                    Object::Array(vec![
                                        Object::Integer(self.toc_pages.get().add(f.page - 1) as i64),
                                        Object::Name(b"XYZ".into()),
                                        Object::Null,
                                        Object::Null,
                                        Object::Null,
                                        Object::Dictionary(dictionary! {
                                            "XYZ" => vec![Object::Null, Object::Null, Object::Null]
                                        }),
                                    ]),
                                );
                            }
                        }
                    }
                }
            }
        }
        document.save(target.as_ref())?;
        Ok(())
    }

    pub fn link_bookmark<P: AsRef<Path>>(&self, param: LinkBookmarkParam<P>) -> anyhow::Result<()> {
        let mut render_data = vec![];
        let base = self.toc_pages.get();

        if let Some(data) = self.visit_render_data.as_ref() {
            let mut data = data.clone();
            data.update_pages(base);
            render_data.push(data);
        }
        if let Some(data) = self.form_render_data.as_ref() {
            let mut data = data.clone();
            data.update_pages(base);
            render_data.push(data);
        }
        let render_file = param.workspace.as_ref().join("bookmark.json");
        fs::write(&render_file, serde_json::to_string(&render_data)?)?;

        let mut cmd = Command::new("cmd");
        cmd.creation_flags(0x08000000);
        // call binary to combine pdf and add outline
        let result = cmd
            .arg("/C")
            .arg(param.acrf_outline_bin.as_ref())
            .arg(param.target.as_ref())
            .arg(render_file)
            .arg(param.target.as_ref())
            .output()?;
        if !result.status.success() {
            let err_message = result.stderr;
            return Err(anyhow!(String::from_utf8(err_message)?));
        }

        Ok(())
    }
}
