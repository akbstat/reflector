use crate::ecrf::ECRF;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Form {
    pub id: usize,
    pub name: String,
    pub page: usize,
    pub order: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Visit {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormVisitBinding {
    pub(crate) parent: usize,
    pub(crate) children: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBStruct {
    pub visit: Vec<Visit>,
    pub form: Vec<Form>,
    pub binding: Vec<FormVisitBinding>,
}

pub enum DBKind {
    ECOLLECT,
}

pub trait DBStructReader<P: AsRef<Path>, E: ECRF> {
    fn read(&self, p: P, ecrf: E) -> anyhow::Result<DBStruct>;
}
