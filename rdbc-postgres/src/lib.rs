//! Postgres RDBC Driver
//!
//! This crate implements an RDBC Driver for the `postgres` crate.
//!
//! The RDBC (Rust DataBase Connectivity) API is loosely based on the ODBC and JDBC standards.
//!
//! ```rust,ignore
//! use rdbc_postgres::PostgresDriver;
//! let driver = PostgresDriver::new();
//! let conn = driver.connect("postgres://postgres@localhost:5433");
//! let stmt = conn.create_statement("SELECT foo FROM bar").unwrap();
//! let rs = stmt.execute_query().unwrap();
//! let mut rs = rs.borrow_mut();
//! while rs.next() {
//!   println!("{}", rs.get_string(1));
//! }
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use postgres;
use postgres::rows::Rows;
use postgres::{Connection, TlsMode};

use postgres::types::{IsNull, Type};
use rdbc;
use rdbc::{ResultSet, Statement};
use std::error::Error;

/// Convert a Postgres error into an RDBC error
fn to_rdbc_err(e: &postgres::error::Error) -> rdbc::Error {
    rdbc::Error::General(format!("{:?}", e))
}

pub struct PostgresDriver {}

impl PostgresDriver {
    pub fn new() -> Self {
        PostgresDriver {}
    }

    pub fn connect(&self, url: &str) -> rdbc::Result<Rc<RefCell<dyn rdbc::Connection>>> {
        postgres::Connection::connect(url, TlsMode::None)
            .map_err(|e| to_rdbc_err(&e))
            .map(|c| {
                Ok(Rc::new(RefCell::new(PConnection::new(c))) as Rc<RefCell<dyn rdbc::Connection>>)
            })?
    }
}

struct PConnection {
    conn: Connection,
}

impl PConnection {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

impl rdbc::Connection for PConnection {
    fn prepare(&mut self, sql: &str) -> rdbc::Result<Rc<RefCell<dyn rdbc::Statement + '_>>> {
        Ok(Rc::new(RefCell::new(PStatement {
            conn: &self.conn,
            sql: sql.to_owned(),
        })) as Rc<RefCell<dyn rdbc::Statement>>)
    }
}

struct PStatement<'a> {
    conn: &'a Connection,
    sql: String,
}

impl<'a> rdbc::Statement for PStatement<'a> {
    fn execute_query(
        &mut self,
        params: &Vec<rdbc::Value>,
    ) -> rdbc::Result<Rc<RefCell<dyn rdbc::ResultSet + '_>>> {
        let params = to_postgres_value(params);
        let params: Vec<&dyn postgres::types::ToSql> = params.iter().map(|v| v.as_ref()).collect();
        self.conn
            .query(&self.sql, params.as_slice())
            .map_err(|e| to_rdbc_err(&e))
            .map(|rows| {
                Rc::new(RefCell::new(PResultSet { i: 0, rows })) as Rc<RefCell<dyn ResultSet>>
            })
    }

    fn execute_update(&mut self, params: &Vec<rdbc::Value>) -> rdbc::Result<usize> {
        let params = to_postgres_value(params);
        let params: Vec<&dyn postgres::types::ToSql> = params.iter().map(|v| v.as_ref()).collect();
        self.conn
            .execute(&self.sql, params.as_slice())
            .map_err(|e| to_rdbc_err(&e))
            .map(|n| n as usize)
    }
}

struct PResultSet {
    i: usize,
    rows: Rows,
}

impl rdbc::ResultSet for PResultSet {
    fn next(&mut self) -> bool {
        if self.i < self.rows.len() {
            self.i = self.i + 1;
            true
        } else {
            false
        }
    }

    fn get_i32(&self, i: usize) -> Option<i32> {
        self.rows.get(self.i - 1).get(i - 1)
    }

    fn get_string(&self, i: usize) -> Option<String> {
        self.rows.get(self.i - 1).get(i - 1)
    }
}

#[derive(Debug)]
struct PostgresValue {
    value: rdbc::Value,
}

impl PostgresValue {
    fn from(value: &rdbc::Value) -> Self {
        Self {
            value: value.clone(),
        }
    }
}

impl postgres::types::ToSql for PostgresValue {
    fn to_sql(&self, ty: &Type, out: &mut Vec<u8>) -> Result<IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized,
    {
        //TODO implement
        unimplemented!()
    }

    fn accepts(ty: &Type) -> bool
    where
        Self: Sized,
    {
        //TODO implement
        unimplemented!()
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut Vec<u8>,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        //TODO implement
        unimplemented!()
    }
}

fn to_postgres_value(values: &Vec<rdbc::Value>) -> Vec<Box<dyn postgres::types::ToSql>> {
    values
        .iter()
        .map(|v| Box::new(PostgresValue::from(v)) as Box<dyn postgres::types::ToSql>)
        .collect()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn execute_query() -> rdbc::Result<()> {
        let driver = PostgresDriver::new();
        let conn = driver.connect("postgres://rdbc:secret@127.0.0.1:5433")?;
        let mut conn = conn.as_ref().borrow_mut();
        let stmt = conn.prepare("SELECT 1")?;
        let mut stmt = stmt.borrow_mut();
        let params = vec![/*rdbc::Value::Int32(1)*/];
        let rs = stmt.execute_query(&params)?;



        let mut rs = rs.as_ref().borrow_mut();

        assert!(rs.next());
        assert_eq!(Some(1), rs.get_i32(1));
        assert!(!rs.next());

        Ok(())
    }
}
