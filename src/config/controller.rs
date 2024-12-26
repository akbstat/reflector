use super::config::ConfigList;
use crate::edc::db::DBStruct;
use nanoid::nanoid;
use std::{
    fs::{create_dir_all, read, remove_file, write, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub struct ConfigController {
    pub root: PathBuf,
}

impl ConfigController {
    pub fn new<P: AsRef<Path>>(root: P) -> anyhow::Result<Self> {
        create_dir_all(root.as_ref().join("config"))?;
        Ok(ConfigController {
            root: root.as_ref().to_path_buf(),
        })
    }

    pub fn list_config(&self) -> anyhow::Result<Vec<ConfigList>> {
        let list = self.config_file_list_path();
        if !list.exists() {
            write(&list, "[]")?;
        }
        let content = read(list)?;
        Ok(serde_json::from_slice::<Vec<ConfigList>>(&content)?)
    }

    pub fn get_config(&self, id: &str) -> anyhow::Result<DBStruct> {
        let content = read(self.config_file_path(id))?;
        Ok(serde_json::from_slice::<DBStruct>(&content)?)
    }

    pub fn save_config(
        &self,
        id: Option<String>,
        name: &str,
        config: &DBStruct,
    ) -> anyhow::Result<String> {
        let mut list = self.list_config()?;
        let id = if let Some(id) = id {
            id.into()
        } else {
            let id = nanoid!();
            list.push(ConfigList {
                id: id.clone(),
                name: name.to_string(),
            });
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(self.config_file_list_path())?
                .write(&serde_json::to_vec(&list)?)?;
            id
        };
        let content = serde_json::to_vec(config)?;
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.config_file_path(&id))?
            .write(&content)?;
        Ok(id)
    }

    pub fn remove_config(&self, id: &str) -> anyhow::Result<()> {
        let mut list = self.list_config()?;
        list.retain(|x| x.id != id);
        let list = serde_json::to_vec(&list)?;
        remove_file(self.config_file_path(id))?;
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.config_file_list_path())?
            .write(&list)?;
        Ok(())
    }

    fn config_file_path(&self, id: &str) -> PathBuf {
        self.root.join("config").join(format!("{}.json", id))
    }

    fn config_file_list_path(&self) -> PathBuf {
        self.root.join("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edc::db::{Form, FormVisitBinding, Visit};

    #[test]
    fn test_config_controller_crud() -> anyhow::Result<()> {
        let temp = Path::new(r"D:\projects\rusty\acrf_outline\.data\ecollect\config_repo");
        let controller = ConfigController::new(temp)?;

        // Test initial empty list
        let list = controller.list_config()?;
        assert!(list.is_empty());

        // Test saving new config
        let mut config = DBStruct {
            visit: vec![
                Visit {
                    id: 0,
                    name: "v0".to_string(),
                    order: 0,
                },
                Visit {
                    id: 1,
                    name: "v1".to_string(),
                    order: 1,
                },
            ],
            form: vec![
                Form {
                    id: 0,
                    name: "f0".to_string(),
                    page: 1,
                    order: 0,
                },
                Form {
                    id: 1,
                    name: "f1".to_string(),
                    page: 2,
                    order: 1,
                },
            ],
            binding: vec![
                FormVisitBinding {
                    parent: 0,
                    children: vec![0],
                },
                FormVisitBinding {
                    parent: 1,
                    children: vec![1],
                },
            ],
        };
        let id = controller.save_config(None, "test config", &config)?;

        // Test list after save
        let list = controller.list_config()?;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "test config");
        assert_eq!(list[0].id, id);

        // Test update config
        config.binding = vec![
            FormVisitBinding {
                parent: 0,
                children: vec![1],
            },
            FormVisitBinding {
                parent: 1,
                children: vec![0],
            },
        ]; // with modified fields
        controller.save_config(Some(id.clone()), "test config updated", &config)?;

        // Test remove config
        controller.remove_config(&id)?;
        let list = controller.list_config()?;
        assert!(list.is_empty());

        Ok(())
    }
}
