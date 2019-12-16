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

use mysql as my;
use rdbc;

use mysql::{Conn, Opts, OptsBuilder};
use std::cell::RefCell;
use std::rc::Rc;

pub struct MySQLDriver {}

impl MySQLDriver {
    pub fn new() -> Self {
        MySQLDriver {}
    }

    pub fn connect(&self, url: &str) -> rdbc::Result<Rc<dyn rdbc::Connection>> {
        let opts = Opts::from_url(&url).expect("DATABASE_URL invalid");
        let mut conn = Conn::new(opts).unwrap();
        Ok(Rc::new(MySQLConnection {
            conn: Rc::new(RefCell::new(conn)),
        }))
    }
}

struct MySQLConnection {
    conn: Rc<RefCell<my::Conn>>,
}

impl rdbc::Connection for MySQLConnection {
    fn create_statement(&self, sql: &str) -> rdbc::Result<Rc<dyn rdbc::Statement>> {
        Ok(Rc::new(MySQLStatement {
            conn: self.conn.clone(),
            sql: sql.to_owned(),
        }))
    }
}

struct MySQLStatement {
    conn: Rc<RefCell<my::Conn>>,
    sql: String,
}

impl rdbc::Statement for MySQLStatement {
    fn execute_query(&self) -> rdbc::Result<Rc<RefCell<dyn rdbc::ResultSet>>> {
        //let x = self.conn.query(sql).unwrap();
        unimplemented!()
    }

    fn execute_update(&self) -> rdbc::Result<usize> {
        unimplemented!()
    }
}

struct MySQLResultSet {}

impl rdbc::ResultSet for MySQLResultSet {
    fn next(&mut self) -> bool {
        unimplemented!()
    }

    fn get_i32(&self, i: usize) -> i32 {
        unimplemented!()
    }

    fn get_string(&self, i: usize) -> String {
        unimplemented!()
    }
}

//

#[cfg(test)]
mod tests {

    use super::*;
    use rdbc::{Connection, ResultSet, Statement};

    //    #[test]
    fn it_works() {
        let driver = MySQLDriver::new();
        let conn = driver
            .connect("mysql://root:password@localhost:3307/mysql")
            .unwrap();
        let stmt = conn.create_statement("SELECT foo FROM bar").unwrap();
        let rs = stmt.execute_query().unwrap();
        let mut rs = rs.borrow_mut();
        while rs.next() {
            println!("{}", rs.get_string(1))
        }
    }
}
