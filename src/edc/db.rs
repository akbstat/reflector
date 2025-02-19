use crate::ecrf::ECRF;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::{ecollect::db::EcollectDBStructReader, rave::db::RaveDBStructReader};

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

#[derive(Debug, Deserialize)]
pub enum DBKind {
    ECollect,
    Rave,
}

pub trait DBStructReader<P: AsRef<Path>> {
    fn read(&self, p: P, ecrf: Box<dyn ECRF>) -> anyhow::Result<DBStruct>;
}

pub fn db_reader<P: AsRef<Path>>(kind: &DBKind) -> Box<dyn DBStructReader<P>> {
    match kind {
        DBKind::ECollect => Box::new(EcollectDBStructReader::new()),
        DBKind::Rave => Box::new(RaveDBStructReader::new()),
    }
}
