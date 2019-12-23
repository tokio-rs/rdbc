//! The RDBC (Rust DataBase Connectivity) API is loosely based on the ODBC and JDBC standards
//! and provides a database agnostic programming interface for executing queries and fetching
//! results.
//!
//! Reference implementation RDBC Drivers exist for Postgres and MySQL.
//!
//! The following example demonstrates how RDBC can be used to run a trivial query against Postgres.
//!
//! ```rust,ignore
//! use rdbc::Value;
//! use rdbc_postgres::PostgresDriver;
//! let driver = PostgresDriver::new();
//! let conn = driver.connect("postgres://postgres:password@localhost:5433").unwrap();
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

/// RDBC Error
#[derive(Debug)]
pub enum Error {
    General(String),
}

#[derive(Debug, Clone)]
pub enum Value {
    Int32(i32),
    UInt32(u32),
    String(String),
    //TODO add other types
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Int32(n) => format!("{}", n),
            Value::UInt32(n) => format!("{}", n),
            Value::String(s) => format!("'{}'", s),
        }
    }
}

/// RDBC Result type
pub type Result<T> = std::result::Result<T, Error>;

/// Represents database driver that can be shared between threads, and can therefore implement
/// a connection pool
pub trait Driver: Sync + Send {
    /// Create a connection to the database. Note that connections are intended to be used
    /// in a single thread since most database connections are not thread-safe
    fn connect(&self, url: &str) -> Result<Rc<RefCell<dyn Connection + 'static>>>;
}

/// Represents a connection to a database
pub trait Connection {
    /// Create a statement for execution
    fn create(&mut self, sql: &str) -> Result<Rc<RefCell<dyn Statement + '_>>>;
    /// Create a prepared statement for execution
    fn prepare(&mut self, sql: &str) -> Result<Rc<RefCell<dyn Statement + '_>>>;
}

/// Represents an executable statement
pub trait Statement {
    /// Execute a query that is expected to return a result set, such as a `SELECT` statement
    fn execute_query(&mut self, params: &[Value]) -> Result<Rc<RefCell<dyn ResultSet + '_>>>;
    /// Execute a query that is expected to update some rows.
    fn execute_update(&mut self, params: &[Value]) -> Result<u64>;
}

/// Result set from executing a query against a statement
pub trait ResultSet {
    // get meta data about this result set
    fn meta_data(&self) -> Result<Rc<dyn ResultSetMetaData>>;
    /// Move the cursor to the next available row if one exists and return true if it does
    fn next(&mut self) -> bool;
    /// Get the i32 value at column `i` (1-based)
    fn get_i32(&self, i: u64) -> Option<i32>;
    /// Get the String value at column `i` (1-based)
    fn get_string(&self, i: u64) -> Option<String>;

    //TODO add accessors for all data types
}

/// Meta data for result set
pub trait ResultSetMetaData {
    fn num_columns(&self) -> u64;
    fn column_name(&self, i: u64) -> String;
    fn column_type(&self, i: u64) -> DataType;
}

/// RDBC Data Types
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DataType {
    Bool,
    Byte,
    Char,
    Short,
    Integer,
    Float,
    Double,
    Decimal,
    Date,
    Time,
    Datetime,
    Utf8,
    Binary,
    //TODO: add more types and research ODBC and JDBC types
}

#[derive(Debug, Clone)]
pub struct Column {
    name: String,
    data_type: DataType,
}

impl Column {
    pub fn new(name: &str, data_type: DataType) -> Self {
        Column {
            name: name.to_owned(),
            data_type,
        }
    }
}

impl ResultSetMetaData for Vec<Column> {
    fn num_columns(&self) -> u64 {
        self.len() as u64
    }

    fn column_name(&self, i: u64) -> String {
        self[i as usize - 1].name.clone()
    }

    fn column_type(&self, i: u64) -> DataType {
        self[i as usize - 1].data_type
    }
}
