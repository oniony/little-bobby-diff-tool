pub mod column;
pub mod column_privilege;
pub mod index;
pub mod privilege;
pub mod routine;
pub mod routine_privilege;
pub mod schema;
pub mod sequence;
pub mod table;
pub mod table_constraint;
pub mod table_privilege;
pub mod table_trigger;
pub mod view;

use sqlx::{Connection, Error, PgConnection};
use crate::db::column::Column;
use crate::db::column_privilege::ColumnPrivilege;
use crate::db::index::Index;
use crate::db::routine::Routine;
use crate::db::routine_privilege::RoutinePrivilege;
use crate::db::table_privilege::TablePrivilege;
use crate::db::table_trigger::TableTrigger;
use crate::db::view::View;

pub struct Database {
    connection: PgConnection
}

impl Database {
    pub async fn connect(url: &str) -> Result<Database, Error> {
        let connection = PgConnection::connect(url).await?;

        Ok(Database {
            connection,
        })
    }

    pub async fn columns(&mut self, schema_names: &[String]) -> Result<Vec<Column>, Error> {
        column::columns(&mut self.connection, schema_names).await
    }

    pub async fn column_privileges(&mut self, schema_names: &[String]) -> Result<Vec<ColumnPrivilege>, Error> {
        column_privilege::column_privileges(&mut self.connection, schema_names).await
    }

    pub async fn indices(&mut self, schema_names: &[String]) -> Result<Vec<Index>, Error> {
        index::indices(&mut self.connection, schema_names).await
    }

    pub async fn routines(&mut self, schema_names: &[String]) -> Result<Vec<Routine>, Error> {
        routine::routines(&mut self.connection, schema_names).await
    }
    
    pub async fn routine_privileges(&mut self, schema_names: &[String]) -> Result<Vec<RoutinePrivilege>, Error> {
        routine_privilege::routine_privileges(&mut self.connection, schema_names).await
    }
    
    pub async fn schemas(&mut self, schema_names: &[String]) -> Result<Vec<schema::Schema>, Error> {
        schema::query(&mut self.connection, &schema_names).await
    }

    pub async fn sequences(&mut self, schema_names: &[String]) -> Result<Vec<sequence::Sequence>, Error> {
        sequence::query(&mut self.connection, &schema_names[..]).await
    }

    pub async fn tables(&mut self, schema_names: &[String]) -> Result<Vec<table::Table>, Error> {
        table::query(&mut self.connection, schema_names).await
    }

    pub async fn table_constraints(&mut self, schema_names: &[String]) -> Result<Vec<table_constraint::TableConstraint>, Error> {
        table_constraint::query(&mut self.connection, schema_names).await
    }
    
    pub async fn table_privileges(&mut self, schema_names: &[String]) -> Result<Vec<TablePrivilege>, Error> {
        table_privilege::query(&mut self.connection, schema_names).await
    }
    
    pub async fn table_triggers(&mut self, schema_names: &[String]) -> Result<Vec<TableTrigger>, Error> {
        table_trigger::query(&mut self.connection, schema_names).await
    }

    pub async fn views(&mut self, schema_names: &[String]) -> Result<Vec<View>, Error> {
        view::query(&mut self.connection, schema_names).await
    }
}
