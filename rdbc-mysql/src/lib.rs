//! MySQL RDBC Driver
//!
//! This crate implements an RDBC Driver for the `mysql` crate.
//!
//! The RDBC (Rust DataBase Connectivity) API is loosely based on the ODBC and JDBC standards.
//!
//! ```rust,no_run
//! use rdbc::*;
//! use rdbc_mysql::MySQLDriver;
//!
//! let driver = MySQLDriver::new();
//! let mut conn = driver.connect("mysql://root:password@localhost:3307/mysql").unwrap();
//! let mut stmt = conn.prepare("SELECT a FROM b WHERE c = ?").unwrap();
//! let mut rs = stmt.execute_query(&[Value::Int32(123)]).unwrap();
//! while rs.next() {
//!   println!("{:?}", rs.get_string(1));
//! }
//! ```

use mysql as my;
use mysql_common::constants::ColumnType;

use sqlparser::dialect::MySqlDialect;
use sqlparser::tokenizer::{Token, Tokenizer, Word};

/// Convert a MySQL error into an RDBC error
fn to_rdbc_err(e: my::error::Error) -> rdbc::Error {
    rdbc::Error::General(e.to_string())
}

fn value_to_rdbc_err(e: my::FromValueError) -> rdbc::Error {
    rdbc::Error::General(e.to_string())
}

pub struct MySQLDriver {}

impl MySQLDriver {
    pub fn new() -> Self {
        MySQLDriver {}
    }
}

impl rdbc::Driver for MySQLDriver {
    fn connect(&self, url: &str) -> rdbc::Result<Box<dyn rdbc::Connection>> {
        let opts = my::Opts::from_url(&url).expect("DATABASE_URL invalid");
        let conn = my::Conn::new(opts).map_err(to_rdbc_err)?;
        Ok(Box::new(MySQLConnection { conn }))
    }
}

struct MySQLConnection {
    conn: my::Conn,
}

impl rdbc::Connection for MySQLConnection {
    fn create(&mut self, sql: &str) -> rdbc::Result<Box<dyn rdbc::Statement + '_>> {
        Ok(Box::new(MySQLStatement {
            conn: &mut self.conn,
            sql: sql.to_owned(),
        }))
    }

    fn prepare<'a>(&'a mut self, sql: &str) -> rdbc::Result<Box<dyn rdbc::Statement + '_>> {
        let stmt = self.conn.prepare(&sql).map_err(to_rdbc_err)?;
        Ok(Box::new(MySQLPreparedStatement { stmt }))
    }
}

struct MySQLStatement<'a> {
    conn: &'a mut my::Conn,
    sql: String,
}

impl<'a> rdbc::Statement for MySQLStatement<'a> {
    fn execute_query(
        &mut self,
        params: &[rdbc::Value],
    ) -> rdbc::Result<Box<dyn rdbc::ResultSet + '_>> {
        let sql = rewrite(&self.sql, params)?;
        let result = self.conn.query(&sql).map_err(to_rdbc_err)?;
        Ok(Box::new(MySQLResultSet { result, row: None }))
    }

    fn execute_update(&mut self, params: &[rdbc::Value]) -> rdbc::Result<u64> {
        let sql = rewrite(&self.sql, params)?;
        self.conn
            .query(&sql)
            .map_err(to_rdbc_err)
            .map(|result| result.affected_rows())
    }
}

struct MySQLPreparedStatement<'a> {
    stmt: my::Stmt<'a>,
}

impl<'a> rdbc::Statement for MySQLPreparedStatement<'a> {
    fn execute_query(
        &mut self,
        params: &[rdbc::Value],
    ) -> rdbc::Result<Box<dyn rdbc::ResultSet + '_>> {
        let result = self
            .stmt
            .execute(to_my_params(params))
            .map_err(to_rdbc_err)?;

        Ok(Box::new(MySQLResultSet { result, row: None }))
    }

    fn execute_update(&mut self, params: &[rdbc::Value]) -> rdbc::Result<u64> {
        self.stmt
            .execute(to_my_params(params))
            .map_err(to_rdbc_err)
            .map(|result| result.affected_rows())
    }
}

pub struct MySQLResultSet<'a> {
    result: my::QueryResult<'a>,
    row: Option<my::Result<my::Row>>,
}

macro_rules! impl_resultset_fns {
    ($($fn: ident -> $ty: ty),*) => {
        $(
            fn $fn(&self, i: u64) -> rdbc::Result<Option<$ty>> {
                match &self.row {
                    Some(Ok(row)) => row
                        .get_opt(i as usize)
                        .expect("we will never `take` the value so the outer `Option` is always `Some`")
                        .map(|v| Some(v))
                        .map_err(value_to_rdbc_err),
                    _ => Ok(None),
                }
            }
        )*
    }
}

impl<'a> rdbc::ResultSet for MySQLResultSet<'a> {
    fn meta_data(&self) -> rdbc::Result<Box<dyn rdbc::ResultSetMetaData>> {
        let meta: Vec<rdbc::Column> = self
            .result
            .columns_ref()
            .iter()
            .map(|c| rdbc::Column::new(&c.name_str(), to_rdbc_type(&c.column_type())))
            .collect();
        Ok(Box::new(meta))
    }

    fn next(&mut self) -> bool {
        self.row = self.result.next();
        self.row.is_some()
    }

    impl_resultset_fns! {
        get_i8 -> i8,
        get_i16 -> i16,
        get_i32 -> i32,
        get_i64 -> i64,
        get_f32 -> f32,
        get_f64 -> f64,
        get_string -> String,
        get_bytes -> Vec<u8>
    }
}

fn to_rdbc_type(t: &ColumnType) -> rdbc::DataType {
    match t {
        ColumnType::MYSQL_TYPE_FLOAT => rdbc::DataType::Float,
        ColumnType::MYSQL_TYPE_DOUBLE => rdbc::DataType::Double,
        ColumnType::MYSQL_TYPE_TINY => rdbc::DataType::Byte,
        ColumnType::MYSQL_TYPE_SHORT => rdbc::DataType::Short,
        ColumnType::MYSQL_TYPE_LONG => rdbc::DataType::Integer,
        ColumnType::MYSQL_TYPE_LONGLONG => rdbc::DataType::Integer, // TODO: 64-bit integer type?
        ColumnType::MYSQL_TYPE_DECIMAL => rdbc::DataType::Decimal,
        ColumnType::MYSQL_TYPE_NEWDECIMAL => rdbc::DataType::Decimal,
        ColumnType::MYSQL_TYPE_STRING => rdbc::DataType::Utf8,
        ColumnType::MYSQL_TYPE_VAR_STRING => rdbc::DataType::Utf8,
        ColumnType::MYSQL_TYPE_VARCHAR => rdbc::DataType::Utf8,
        ColumnType::MYSQL_TYPE_TINY_BLOB => rdbc::DataType::Binary,
        ColumnType::MYSQL_TYPE_MEDIUM_BLOB => rdbc::DataType::Binary,
        ColumnType::MYSQL_TYPE_LONG_BLOB => rdbc::DataType::Binary,
        ColumnType::MYSQL_TYPE_BLOB => rdbc::DataType::Binary,
        ColumnType::MYSQL_TYPE_BIT => rdbc::DataType::Bool,
        ColumnType::MYSQL_TYPE_DATE => rdbc::DataType::Date,
        ColumnType::MYSQL_TYPE_TIME => rdbc::DataType::Time,
        ColumnType::MYSQL_TYPE_TIMESTAMP => rdbc::DataType::Datetime, // TODO: Data type for timestamps in UTC?
        ColumnType::MYSQL_TYPE_DATETIME => rdbc::DataType::Datetime,
        mysql_datatype => todo!("Datatype not currently supported: {:?}", mysql_datatype),
    }
}

fn to_my_value(v: &rdbc::Value) -> my::Value {
    match v {
        rdbc::Value::Int32(n) => my::Value::Int(*n as i64),
        rdbc::Value::UInt32(n) => my::Value::Int(*n as i64),
        rdbc::Value::String(s) => my::Value::from(s),
        //TODO all types
    }
}

/// Convert RDBC parameters to MySQL parameters
fn to_my_params(params: &[rdbc::Value]) -> my::Params {
    my::Params::Positional(params.iter().map(|v| to_my_value(v)).collect())
}

fn rewrite(sql: &str, params: &[rdbc::Value]) -> rdbc::Result<String> {
    let dialect = MySqlDialect {};
    let mut tokenizer = Tokenizer::new(&dialect, sql);
    tokenizer
        .tokenize()
        .and_then(|tokens| {
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
                            keyword: sqlparser::dialect::keywords::Keyword::NoKeyword,
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

            Ok(sql)
        })
        .map_err(|e| rdbc::Error::General(format!("{:?}", e)))
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::sync::Arc;

    #[test]
    fn execute_query() -> rdbc::Result<()> {
        execute("DROP TABLE IF EXISTS test", &vec![])?;
        execute("CREATE TABLE test (a INT NOT NULL)", &vec![])?;
        execute(
            "INSERT INTO test (a) VALUES (?)",
            &vec![rdbc::Value::Int32(123)],
        )?;

        let driver: Arc<dyn rdbc::Driver> = Arc::new(MySQLDriver::new());
        let mut conn = driver.connect("mysql://root:secret@127.0.0.1:3307/mysql")?;
        let mut stmt = conn.prepare("SELECT a FROM test")?;
        let mut rs = stmt.execute_query(&vec![])?;
        assert!(rs.next());
        assert_eq!(Some(123), rs.get_i32(0)?);
        assert!(!rs.next());

        Ok(())
    }

    fn execute(sql: &str, values: &Vec<rdbc::Value>) -> rdbc::Result<u64> {
        println!("Executing '{}' with {} params", sql, values.len());
        let driver: Arc<dyn rdbc::Driver> = Arc::new(MySQLDriver::new());
        let mut conn = driver.connect("mysql://root:secret@127.0.0.1:3307/mysql")?;
        let mut stmt = conn.create(sql)?;
        stmt.execute_update(values)
    }
}
