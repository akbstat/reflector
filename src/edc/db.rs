use crate::ecrf::ECRF;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Form {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) page: usize,
    pub(crate) order: i32,
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
    pub(crate) visit: Vec<Visit>,
    pub(crate) form: Vec<Form>,
    pub(crate) binding: Vec<FormVisitBinding>,
}

pub enum DBKind {
    ECOLLECT,
}

pub trait DBStructReader<P: AsRef<Path>, E: ECRF> {
    fn read(&self, p: P, ecrf: E) -> anyhow::Result<DBStruct>;
}
