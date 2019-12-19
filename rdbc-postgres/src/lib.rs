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

use rdbc;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use postgres;
use postgres::rows::Rows;
use postgres::{Connection, TlsMode};
use rdbc::ResultSet;

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
    conn: Rc<Connection>,
}

impl PConnection {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Rc::new(conn),
        }
    }
}

impl rdbc::Connection for PConnection {
    fn execute_query(
        &mut self,
        sql: &str,
        _params: HashMap<String, rdbc::Value>,
    ) -> rdbc::Result<Rc<RefCell<dyn ResultSet + '_>>> {
        self.conn
            .query(sql, &[])
            .map_err(|e| to_rdbc_err(&e))
            .map(|rows| {
                Rc::new(RefCell::new(PResultSet { i: 0, rows })) as Rc<RefCell<dyn ResultSet>>
            })
    }

    fn execute_update(
        &mut self,
        sql: &str,
        _params: HashMap<String, rdbc::Value>,
    ) -> rdbc::Result<usize> {
        self.conn
            .execute(sql, &[])
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn execute_query() -> rdbc::Result<()> {
        let driver = PostgresDriver::new();
        let conn = driver.connect("postgres://rdbc:secret@127.0.0.1:5433")?;
        let mut conn = conn.as_ref().borrow_mut();
        let rs = conn.execute_query("SELECT 1", HashMap::new())?;
        let mut rs = rs.as_ref().borrow_mut();
        while rs.next() {
            println!("{:?}", rs.get_i32(1))
        }
        Ok(())
    }
}
