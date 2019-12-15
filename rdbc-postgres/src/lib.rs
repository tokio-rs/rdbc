//! Postgres RDBC Driver
//!
//! This crate implements an RDBC Driver for the `postgres` crate.
//!
//! The RDBC (Rust DataBase Connectivity) API is loosely based on the ODBC and JDBC standards.
//!
//! ```
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

use std::rc::Rc;
use std::cell::RefCell;

use postgres::{Connection, TlsMode};
use rdbc::ResultSet;
use postgres::rows::Rows;

struct PostgresDriver {}

impl PostgresDriver {
    pub fn new() -> Self {
        PostgresDriver {}
    }

    pub fn connect(&self, url: &str) -> Rc<rdbc::Connection> {
        let conn = postgres::Connection::connect(url, TlsMode::None).unwrap();
        Rc::new(PConnection::new(conn))
    }
}

struct PConnection {
    conn: Rc<Connection>
}

impl PConnection {
    pub fn new(conn: Connection) -> Self {
        Self { conn: Rc::new(conn) }
    }
}

impl rdbc::Connection for PConnection {

    fn create_statement(&self, sql: &str) -> rdbc::Result<Rc<dyn rdbc::Statement>> {
        Ok(Rc::new(PStatement {
            conn: self.conn.clone(),
            sql: sql.to_owned()
        }))
    }

}

struct PStatement {
    conn: Rc<Connection>,
    sql: String
}

impl rdbc::Statement for PStatement {

    fn execute_query(&self) -> rdbc::Result<Rc<RefCell<dyn ResultSet>>> {
        let rows: Rows = self.conn.query(&self.sql, &[]).unwrap();
        Ok(Rc::new(RefCell::new(PResultSet { i: 0, rows })))
    }

    fn execute_update(&self) -> rdbc::Result<usize> {
        unimplemented!()
    }
}

struct PResultSet {
    i: usize,
    rows: Rows
}

impl rdbc::ResultSet for PResultSet {

    fn next(&mut self) -> bool {
        if self.i+1 < self.rows.len() {
            self.i = self.i + 1;
            true
        } else {
            false
        }
    }

    fn get_i32(&self, i: usize) -> i32 {
        self.rows.get(self.i).get(i)
    }

    fn get_string(&self, i: usize) -> String {
        self.rows.get(self.i).get(i)
    }

}

#[cfg(test)]
mod tests {

    use super::*;
    use rdbc::{Connection, Statement, ResultSet};

    //#[test]
    fn it_works() {
        let driver = PostgresDriver::new();
        let conn = driver.connect("postgres://postgres@localhost:5433");
        let stmt = conn.create_statement("SELECT foo FROM bar").unwrap();
        let rs = stmt.execute_query().unwrap();
        let mut rs = rs.borrow_mut();
        while rs.next() {
            println!("{}", rs.get_string(1))
        }
    }
}
