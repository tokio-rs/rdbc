//! MySQL RDBC Driver
//!
//! This crate implements an RDBC Driver for the `mysql` crate.
//!
//! The RDBC (Rust DataBase Connectivity) API is loosely based on the ODBC and JDBC standards.
//!
//! ```rust,ignore
//! use rdbc_mysql::MySQLDriver;
//! let driver = MySQLDriver::new();
//! let conn = driver.connect("mysql://root:password@localhost:3307/mysql").unwrap();
//! let stmt = conn.create_statement("SELECT foo FROM bar").unwrap();
//! let rs = stmt.execute_query().unwrap();
//! let mut rs = rs.borrow_mut();
//! while rs.next() {
//!   println!("{}", rs.get_string(1));
//! }
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use mysql as my;
use rdbc;

/// Convert a MySQL error into an RDBC error
fn to_rdbc_err(e: &my::error::Error) -> rdbc::Error {
    rdbc::Error::General(format!("{:?}", e))
}

pub struct MySQLDriver {}

impl MySQLDriver {
    pub fn new() -> Self {
        MySQLDriver {}
    }

    pub fn connect(&self, url: &str) -> rdbc::Result<Rc<RefCell<dyn rdbc::Connection + 'static>>> {
        let opts = my::Opts::from_url(&url).expect("DATABASE_URL invalid");
        my::Conn::new(opts)
            .map_err(|e| to_rdbc_err(&e))
            .map(|conn| Rc::new(RefCell::new(MySQLConnection { conn })) as Rc<RefCell<dyn rdbc::Connection>>)
    }
}

pub struct MySQLConnection {
    conn: my::Conn,
}

impl rdbc::Connection for MySQLConnection {
    fn execute_query(&mut self, sql: &str) -> rdbc::Result<Rc<RefCell<dyn rdbc::ResultSet + '_>>> {
        self.conn.query(sql)
            .map_err(|e| to_rdbc_err(&e))
            .map(|result| Rc::new(RefCell::new(MySQLResultSet { result, row: None })) as Rc<RefCell<dyn rdbc::ResultSet>>)
    }

    fn execute_update(&mut self, sql: &str) -> rdbc::Result<usize> {
        self.conn.query(sql)
            .map_err(|e| to_rdbc_err(&e))
            .map(|result| result.affected_rows() as usize)
    }

}

pub struct MySQLResultSet<'a> {
    result: my::QueryResult<'a>,
    row: Option<my::Result<my::Row>>,
}

impl<'a> rdbc::ResultSet for MySQLResultSet<'a> {
    fn next(&mut self) -> bool {
        self.row = self.result.next();
        self.row.is_some()
    }

    fn get_i32(&self, i: usize) -> Option<i32> {
        match &self.row {
            Some(Ok(row)) => row.get(i - 1),
            _ => None,
        }
    }

    fn get_string(&self, i: usize) -> Option<String> {
        match &self.row {
            Some(Ok(row)) => row.get(i - 1),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn execute_query() -> rdbc::Result<()> {
        let driver = MySQLDriver::new();

        let conn = driver
            .connect("mysql://root:secret@127.0.0.1:3307")?;

        let mut conn = conn.as_ref().borrow_mut();

        let rs = conn.execute_query("SELECT 1")?;
        let mut rs = rs.borrow_mut();

        while rs.next() {
            println!("{:?}", rs.get_i32(1))
        }

        Ok(())
    }
}
