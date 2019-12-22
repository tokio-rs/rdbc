//! MySQL RDBC Driver
//!
//! This crate implements an RDBC Driver for the `mysql` crate.
//!
//! The RDBC (Rust DataBase Connectivity) API is loosely based on the ODBC and JDBC standards.
//!
//! ```rust,ignore
//! use rdbc::Value;
//! use rdbc_mysql::MySQLDriver;
//! let driver = MySQLDriver::new();
//! let conn = driver.connect("mysql://root:password@localhost:3307/mysql").unwrap();
//! let mut conn = conn.borrow_mut();
//! let stmt = conn.prepare("SELECT a FROM b WHERE c = ?").unwrap();
//! let mut stmt = stmt.borrow_mut();
//! let rs = stmt.execute_query(&vec![Value::Int32(123)]).unwrap();
//! let mut rs = rs.borrow_mut();
//! while rs.next() {
//!   println!("{:?}", rs.get_string(1));
//! }
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use mysql as my;

use rdbc;

use sqlparser::dialect::MySqlDialect;
use sqlparser::tokenizer::{Token, Tokenizer, Word};

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
    fn create(&mut self, sql: &str) -> rdbc::Result<Rc<RefCell<dyn rdbc::Statement + '_>>> {
        Ok(Rc::new(RefCell::new(MySQLStatement {
            conn: &mut self.conn,
            sql: sql.to_owned(),
        })) as Rc<RefCell<dyn rdbc::Statement>>)
    }

    fn prepare(&mut self, sql: &str) -> rdbc::Result<Rc<RefCell<dyn rdbc::Statement + '_>>> {
        let stmt = self.conn.prepare(&sql).unwrap();
        Ok(Rc::new(RefCell::new(MySQLPreparedStatement { stmt }))
            as Rc<RefCell<dyn rdbc::Statement>>)
    }
}

struct MySQLStatement<'a> {
    conn: &'a mut my::Conn,
    sql: String,
}

impl<'a> rdbc::Statement for MySQLStatement<'a> {
    fn execute_query(
        &mut self,
        params: &Vec<rdbc::Value>,
    ) -> rdbc::Result<Rc<RefCell<dyn rdbc::ResultSet + '_>>> {
        let sql = rewrite(&self.sql, params);
        self.conn
            .query(&sql)
            .map_err(|e| to_rdbc_err(&e))
            .map(|result| {
                Rc::new(RefCell::new(MySQLResultSet { result, row: None }))
                    as Rc<RefCell<dyn rdbc::ResultSet>>
            })
    }

    fn execute_update(&mut self, params: &Vec<rdbc::Value>) -> rdbc::Result<u64> {
        let sql = rewrite(&self.sql, params);
        self.conn
            .query(&sql)
            .map_err(|e| to_rdbc_err(&e))
            .map(|result| result.affected_rows())
    }
}

struct MySQLPreparedStatement<'a> {
    stmt: my::Stmt<'a>,
}

impl<'a> rdbc::Statement for MySQLPreparedStatement<'a> {
    fn execute_query(
        &mut self,
        params: &Vec<rdbc::Value>,
    ) -> rdbc::Result<Rc<RefCell<dyn rdbc::ResultSet + '_>>> {
        self.stmt
            .execute(to_my_params(params))
            .map_err(|e| to_rdbc_err(&e))
            .map(|result| {
                Rc::new(RefCell::new(MySQLResultSet { result, row: None }))
                    as Rc<RefCell<dyn rdbc::ResultSet>>
            })
    }

    fn execute_update(&mut self, params: &Vec<rdbc::Value>) -> rdbc::Result<u64> {
        self.stmt
            .execute(to_my_params(params))
            .map_err(|e| to_rdbc_err(&e))
            .map(|result| result.affected_rows())
    }
}

pub struct MySQLResultSet<'a> {
    result: my::QueryResult<'a>,
    row: Option<my::Result<my::Row>>,
}

impl<'a> rdbc::ResultSet for MySQLResultSet<'a> {

    fn meta_data(&self) -> rdbc::Result<Rc<dyn rdbc::ResultSetMetaData>> {
        unimplemented!()
    }

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

/// Convert RDBC parameters to MySQL parameters
fn to_my_params(params: &Vec<rdbc::Value>) -> my::Params {
    my::Params::Positional(params.iter().map(|v| to_my_value(v)).collect())
}

fn rewrite(sql: &str, params: &Vec<rdbc::Value>) -> String {
    let dialect = MySqlDialect {};
    let mut tokenizer = Tokenizer::new(&dialect, sql);
    let tokens = tokenizer.tokenize().unwrap();
    let mut i = 0;

    let tokens: Vec<Token> = tokens
        .iter()
        .map(|t| match t {
            Token::Char(c) if *c == '?' => {
                let param = &params[i];
                i += 1;
                Token::Word(Word {
                    value: param.to_string(),
                    quote_style: None,
                    keyword: "".to_owned(),
                })
            }
            _ => t.clone(),
        })
        .collect();

    let sql = tokens
        .iter()
        .map(|t| format!("{}", t))
        .collect::<Vec<String>>()
        .join("");

    sql
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn execute_query() -> rdbc::Result<()> {
        execute("DROP TABLE IF EXISTS test", &vec![])?;
        execute("CREATE TABLE test (a INT NOT NULL)", &vec![])?;
        execute(
            "INSERT INTO test (a) VALUES (?)",
            &vec![rdbc::Value::Int32(123)],
        )?;

        let driver = MySQLDriver::new();
        let conn = driver.connect("mysql://root:secret@127.0.0.1:3307/mysql")?;
        let mut conn = conn.as_ref().borrow_mut();
        let stmt = conn.prepare("SELECT a FROM test")?;
        let mut stmt = stmt.borrow_mut();
        let rs = stmt.execute_query(&vec![])?;

        let mut rs = rs.as_ref().borrow_mut();

        assert!(rs.next());
        assert_eq!(Some(123), rs.get_i32(1));
        assert!(!rs.next());

        Ok(())
    }

    fn execute(sql: &str, values: &Vec<rdbc::Value>) -> rdbc::Result<u64> {
        println!("Executing '{}' with {} params", sql, values.len());
        let driver = MySQLDriver::new();
        let conn = driver.connect("mysql://root:secret@127.0.0.1:3307/mysql")?;
        let mut conn = conn.as_ref().borrow_mut();
        let stmt = conn.create(sql)?;
        let mut stmt = stmt.borrow_mut();
        stmt.execute_update(values)
    }
}
