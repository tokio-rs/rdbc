//! The RDBC (Rust DataBase Connectivity) API is loosely based on the ODBC and JDBC standards
//! and provides a database agnostic programming interface for executing queries and fetching
//! results.
//!
//! Reference implementation RDBC Drivers exist for Postgres and MySQL.
//!
//! The following example demonstrates how RDBC can be used to run a trivial query against Postgres.
//!
//! ```rust,ignore
//! let driver = PostgresDriver::new();
//! let conn = driver.connect("postgres://postgres@localhost:5433");
//! let stmt = conn.create_statement("SELECT foo FROM bar").unwrap();
//! let rs = stmt.execute_query().unwrap();
//! let mut rs = rs.borrow_mut();
//! while rs.next() {
//!   println!("{}", rs.get_string(1));
//! }
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// RDBC Error
#[derive(Debug)]
pub enum Error {
    General(String),
}

#[derive(Debug)]
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
            Value::String(s) => s.clone(),
        }
    }
}

/// RDBC Result type
pub type Result<T> = std::result::Result<T, Error>;

/// Represents a connection to a database
pub trait Connection {
    /// Prepare a SQL statement for execution
    fn prepare(&mut self, sql: &str) -> Result<Rc<RefCell<dyn Statement + '_>>>;
}

pub trait Statement {
    /// Execute a query that is expected to return a result set, such as a `SELECT` statement
    fn execute_query(
        &mut self,
        params: &HashMap<String, Value>,
    ) -> Result<Rc<RefCell<dyn ResultSet + '_>>>;

    /// Execute a query that is expected to update some rows.
    fn execute_update(&mut self, params: &HashMap<String, Value>) -> Result<usize>;
}

/// Result set from executing a query against a statement
pub trait ResultSet {
    /// Move the cursor to the next available row if one exists and return true if it does
    fn next(&mut self) -> bool;
    /// Get the i32 value at column `i` (1-based)
    fn get_i32(&self, i: usize) -> Option<i32>;
    /// Get the String value at column `i` (1-based)
    fn get_string(&self, i: usize) -> Option<String>;
    //TODO add accessors for all data types
}

/// Simplistic code to replace named parameters in a query
pub fn replace_params(sql: &str, params: &HashMap<String, Value>) -> String {
    let mut sql = sql.to_owned();
    params.iter().for_each(|(k, v)| {
        let param_name = format!(":{}", k);
        let param_value = v.to_string();
        sql = sql.replace(&param_name, &param_value);
    });
    sql
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let sql = "INSERT foo (id, name) VALUES (:id, :name)";
        let mut params = HashMap::new();
        params.insert("id".to_owned(), Value::Int32(123));
        params.insert("name".to_owned(), Value::String("Bob".to_owned()));
        assert_eq!(
            "INSERT foo (id, name) VALUES (123, Bob)".to_owned(),
            replace_params(&sql, &params)
        );
    }
}
