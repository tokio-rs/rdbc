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
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::rc::Rc;

use mysql as my;
use rdbc;
use twox_hash::XxHash64;

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
            .map(|conn| {
                Rc::new(RefCell::new(MySQLConnection { conn })) as Rc<RefCell<dyn rdbc::Connection>>
            })
    }
}

struct MySQLConnection {
    conn: my::Conn,
}

impl rdbc::Connection for MySQLConnection {
    fn prepare(&mut self, sql: &str) -> rdbc::Result<Rc<RefCell<dyn rdbc::Statement + '_>>> {
        let stmt = self.conn.prepare(&sql).unwrap();
        Ok(Rc::new(RefCell::new(MySQLStatement {
            stmt,
            sql: sql.to_owned(),
        })) as Rc<RefCell<dyn rdbc::Statement>>)
    }
}

struct MySQLStatement<'a> {
    stmt: my::Stmt<'a>,
    sql: String,
}

impl<'a> rdbc::Statement for MySQLStatement<'a> {
    fn execute_query(
        &mut self,
        params: &HashMap<String, rdbc::Value>,
    ) -> rdbc::Result<Rc<RefCell<dyn rdbc::ResultSet + '_>>> {
        let mysql_params = to_my_params(params);
        self.stmt
            .execute(my::Params::Named(mysql_params))
            .map_err(|e| to_rdbc_err(&e))
            .map(|result| {
                Rc::new(RefCell::new(MySQLResultSet { result, row: None }))
                    as Rc<RefCell<dyn rdbc::ResultSet>>
            })
    }

    fn execute_update(&mut self, params: &HashMap<String, rdbc::Value>) -> rdbc::Result<usize> {
        let mysql_params = to_my_params(params);
        self.stmt
            .execute(my::Params::Named(mysql_params))
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

fn to_my_value(v: &rdbc::Value) -> my::Value {
    match v {
        rdbc::Value::Int32(n) => my::Value::Int(*n as i64),
        rdbc::Value::UInt32(n) => my::Value::Int(*n as i64),
        rdbc::Value::String(s) => my::Value::from(s),
    }
}

type MyBuildHasher = BuildHasherDefault<XxHash64>;

/// Convert RDBC parameters to MySQL parameters
fn to_my_params(
    params: &HashMap<String, rdbc::Value>,
) -> HashMap<String, my::Value, MyBuildHasher> {
    let mut mysql_params = HashMap::<String, my::Value, MyBuildHasher>::default();
    params.iter().for_each(|(k, v)| {
        mysql_params.insert(k.clone(), to_my_value(v));
    });
    mysql_params
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn execute_query() -> rdbc::Result<()> {
        let driver = MySQLDriver::new();

        let conn = driver.connect("mysql://root:secret@127.0.0.1:3307")?;

        let mut conn = conn.as_ref().borrow_mut();

        let stmt = conn.prepare("SELECT :number")?;
        let mut stmt = stmt.borrow_mut();

        let mut params = HashMap::default();
        params.insert("number".to_owned(), rdbc::Value::Int32(1));

        let rs = stmt.execute_query(&params)?;

        let mut rs = rs.borrow_mut();

        assert!(rs.next());
        assert_eq!(Some(1), rs.get_i32(1));
        assert!(!rs.next());

        Ok(())
    }
}
