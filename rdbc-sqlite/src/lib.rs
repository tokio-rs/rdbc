//! SQLite RDBC Driver
//!
//! This crate implements an RDBC Driver for the `rusqlite` crate.
//!
//! The RDBC (Rust DataBase Connectivity) API is loosely based on the ODBC and JDBC standards.
//!
//! ```rust
//! use std::sync::Arc;
//! use rdbc::{self, Value};
//! use rdbc_sqlite::SqliteDriver;
//! let driver: Arc<dyn rdbc::Driver> = Arc::new(SqliteDriver::new());
//! let mut conn = driver.connect("").unwrap();
//! let stmt = conn.prepare("CREATE TABLE test (a INT NOT NULL)").unwrap().execute_update(&[]).unwrap();
//! let stmt = conn.prepare("INSERT INTO test (a) VALUES (?)").unwrap().execute_update(&[rdbc::Value::Int32(123)]).unwrap();
//! let mut stmt = conn.prepare("SELECT a FROM test").unwrap();
//! let mut rs = stmt.execute_query(&[]).unwrap();
//! assert!(rs.next());
//! assert_eq!(Some(123), rs.get_i32(0).unwrap());
//! ```

use async_trait::async_trait;
use fallible_streaming_iterator::FallibleStreamingIterator;
use rusqlite::Rows;

/// Convert a Sqlite error into an RDBC error
fn to_rdbc_err(e: rusqlite::Error) -> rdbc::Error {
    rdbc::Error::General(format!("{:?}", e))
}

pub struct SqliteDriver {}

impl SqliteDriver {
    pub fn new() -> Self {
        SqliteDriver {}
    }
}

#[async_trait]
impl rdbc::Driver for SqliteDriver {
    async fn connect(&self, _url: &str) -> rdbc::Result<Box<dyn rdbc::Connection>> {
        let c = rusqlite::Connection::open_in_memory().map_err(to_rdbc_err)?;
        Ok(Box::new(SConnection::new(c)))
    }
}

struct SConnection {
    conn: rusqlite::Connection,
}

impl SConnection {
    pub fn new(conn: rusqlite::Connection) -> Self {
        Self { conn }
    }
}

impl rdbc::Connection for SConnection {
    fn create(&mut self, sql: &str) -> rdbc::Result<Box<dyn rdbc::Statement + '_>> {
        self.prepare(sql)
    }

    fn prepare(&mut self, sql: &str) -> rdbc::Result<Box<dyn rdbc::Statement + '_>> {
        let stmt = self.conn.prepare(sql).map_err(to_rdbc_err)?;
        Ok(Box::new(SStatement { stmt }))
    }
}

struct SStatement<'a> {
    stmt: rusqlite::Statement<'a>,
}

impl<'a> rdbc::Statement for SStatement<'a> {
    fn execute_query(
        &mut self,
        params: &[rdbc::Value],
    ) -> rdbc::Result<Box<dyn rdbc::ResultSet + '_>> {
        let params = Values(params);
        let rows = self.stmt.query(&params).map_err(to_rdbc_err)?;
        Ok(Box::new(SResultSet { rows }))
    }

    fn execute_update(&mut self, params: &[rdbc::Value]) -> rdbc::Result<u64> {
        let params = Values(params);
        return self
            .stmt
            .execute(&params)
            .map_err(to_rdbc_err)
            .map(|n| n as u64);
    }
}

macro_rules! impl_resultset_fns {
    ($($fn: ident -> $ty: ty),*) => {
        $(
            fn $fn(&self, i: u64) -> rdbc::Result<Option<$ty>> {
                self.rows
                    .get()
                    .unwrap()
                    .get(i as usize)
                    .map_err(to_rdbc_err)
            }
        )*
    }
}

struct SResultSet<'stmt> {
    rows: Rows<'stmt>,
}

impl<'stmt> rdbc::ResultSet for SResultSet<'stmt> {
    fn meta_data(&self) -> rdbc::Result<Box<dyn rdbc::ResultSetMetaData>> {
        let meta: Vec<rdbc::Column> = self
            .rows
            .columns()
            .unwrap()
            .iter()
            .map(|c| rdbc::Column::new(c.name(), to_rdbc_type(c.decl_type())))
            .collect();
        Ok(Box::new(meta))
    }

    fn next(&mut self) -> bool {
        self.rows.next().unwrap().is_some()
    }

    fn get_f32(&self, _i: u64) -> rdbc::Result<Option<f32>> {
        Err(rdbc::Error::General("f32 not supported".to_owned()))
    }

    impl_resultset_fns! {
        get_i8 -> i8,
        get_i16 -> i16,
        get_i32 -> i32,
        get_i64 -> i64,
        get_f64 -> f64,
        get_string -> String,
        get_bytes -> Vec<u8>
    }
}

fn to_rdbc_type(t: Option<&str>) -> rdbc::DataType {
    //TODO implement for real
    match t {
        Some("INT") => rdbc::DataType::Integer,
        _ => rdbc::DataType::Utf8,
    }
}

struct Values<'a>(&'a [rdbc::Value]);
struct ValuesIter<'a>(std::slice::Iter<'a, rdbc::Value>);

impl<'a> IntoIterator for &'a Values<'a> {
    type IntoIter = ValuesIter<'a>;
    type Item = &'a dyn rusqlite::types::ToSql;

    fn into_iter(self) -> ValuesIter<'a> {
        ValuesIter(self.0.iter())
    }
}
impl<'a> Iterator for ValuesIter<'a> {
    type Item = &'a dyn rusqlite::types::ToSql;

    fn next(&mut self) -> Option<&'a dyn rusqlite::types::ToSql> {
        self.0.next().map(|v| match v {
            rdbc::Value::String(ref s) => s as &dyn rusqlite::types::ToSql,
            rdbc::Value::Int32(ref n) => n as &dyn rusqlite::types::ToSql,
            rdbc::Value::UInt32(ref n) => n as &dyn rusqlite::types::ToSql,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdbc::{Connection, DataType};
    use std::sync::Arc;

    #[async_test]
    fn execute_query() -> rdbc::Result<()> {
        let driver: Arc<dyn rdbc::Driver> = Arc::new(SqliteDriver::new());
        let url = "";
        let mut conn = driver.connect(url).await?;
        execute(&mut *conn, "DROP TABLE IF EXISTS test", &vec![])?;
        execute(&mut *conn, "CREATE TABLE test (a INT NOT NULL)", &vec![])?;
        execute(
            &mut *conn,
            "INSERT INTO test (a) VALUES (?)",
            &vec![rdbc::Value::Int32(123)],
        )?;

        let mut stmt = conn.prepare("SELECT a FROM test")?;
        let mut rs = stmt.execute_query(&vec![])?;

        let meta = rs.meta_data()?;
        assert_eq!(1, meta.num_columns());
        assert_eq!("a".to_owned(), meta.column_name(0));
        assert_eq!(DataType::Integer, meta.column_type(0));

        assert!(rs.next());
        assert_eq!(Some(123), rs.get_i32(0)?);
        assert!(!rs.next());

        Ok(())
    }

    fn execute(
        conn: &mut dyn Connection,
        sql: &str,
        values: &Vec<rdbc::Value>,
    ) -> rdbc::Result<u64> {
        println!("Executing '{}' with {} params", sql, values.len());
        let mut stmt = conn.prepare(sql)?;
        stmt.execute_update(values)
    }
}
