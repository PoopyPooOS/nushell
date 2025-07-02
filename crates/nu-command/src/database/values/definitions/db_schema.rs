use super::db_table::DbTable;

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DbSchema {
    pub name: String,
    pub tables: Vec<DbTable>,
}
