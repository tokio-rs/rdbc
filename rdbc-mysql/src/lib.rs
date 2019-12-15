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
//! let mut stmt = conn.create_statement("SELECT foo FROM bar").unwrap();
//! let rs = stmt.execute_query().unwrap();
//! let mut rs = rs.borrow_mut();
//! while rs.next() {
//!   println!("{}", rs.get_string(1));
//! }
//! ```

use rdbc;
use mysql as my;

use std::rc::Rc;
use std::cell::RefCell;

pub struct MySQLDriver {}

impl MySQLDriver {

    pub fn new() -> Self {
        MySQLDriver {}
    }

    pub fn connect(&self, url: &str) -> rdbc::Result<Rc<dyn rdbc::Connection>> {
        let pool = my::Pool::new(url).unwrap();
        Ok(Rc::new(MySQLConnection { conn: Rc::new(RefCell::new(pool.get_conn().unwrap())) }))
    }

}

struct MySQLConnection {
    conn: Rc<RefCell<my::PooledConn>>
}


impl rdbc::Connection for MySQLConnection {
    fn create_statement(&self, sql: &str) -> rdbc::Result<Rc<RefCell<dyn rdbc::Statement>>> {
        Ok(Rc::new(RefCell::new(MySQLStatement { conn: self.conn.clone(), sql: sql.to_owned() })))
    }
}

struct MySQLStatement {
    conn: Rc<RefCell<my::PooledConn>>,
    sql: String

}

impl rdbc::Statement for MySQLStatement {
    fn execute_query(&mut self) -> rdbc::Result<Rc<RefCell<dyn rdbc::ResultSet>>> {
        let mut conn = self.conn.borrow_mut();
        conn.query(&self.sql);
        unimplemented!()
    }

    fn execute_update(&mut self) -> rdbc::Result<usize> {
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
    use rdbc::{Connection, Statement, ResultSet};
    use std::borrow::BorrowMut;

    //    #[test]
    fn it_works() {
        let driver = MySQLDriver::new();
        let conn = driver.connect("mysql://root:password@localhost:3307/mysql").unwrap();
        let stmt = conn.create_statement("SELECT foo FROM bar").unwrap();
        let mut stmt = stmt.borrow_mut();
        let rs = stmt.execute_query().unwrap();
        let mut rs = rs.borrow_mut();
        while rs.next() {
            println!("{}", rs.get_string(1))
        }
    }
}
