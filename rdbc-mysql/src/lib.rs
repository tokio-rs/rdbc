use rdbc;
use mysql as my;

use std::rc::Rc;
use std::cell::RefCell;

struct MySQLDriver {}

impl MySQLDriver {

    pub fn new() -> Self {
        MySQLDriver {}
    }

    pub fn connect(&self, url: &str) -> rdbc::Result<Rc<dyn rdbc::Connection>> {
        let pool = my::Pool::new(url).unwrap();
        Ok(Rc::new(MySQLConnection { pool }))
    }

}

struct MySQLConnection {
    pool: my::Pool
}


impl rdbc::Connection for MySQLConnection {
    fn create_statement(&self, sql: &str) -> rdbc::Result<Rc<dyn rdbc::Statement>> {
        unimplemented!()
    }
}

struct MySQLStatement {}

impl rdbc::Statement for MySQLStatement {
    fn execute_query(&self) -> rdbc::Result<Rc<RefCell<dyn rdbc::ResultSet>>> {
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
    use rdbc::{Connection, Statement, ResultSet};

    #[test]
    fn it_works() {
        let driver = MySQLDriver::new();
        let conn = driver.connect("mysql://root:password@localhost:3307/mysql").unwrap();
        let stmt = conn.create_statement("SELECT foo FROM bar").unwrap();
        let rs = stmt.execute_query().unwrap();
        let mut rs = rs.borrow_mut();
        while rs.next() {
            println!("{}", rs.get_string(1))
        }
    }
}
