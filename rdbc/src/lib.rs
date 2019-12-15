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

use std::rc::Rc;
use std::cell::RefCell;

/// RDBC Result type
pub type Result<T> = std::result::Result<T, String>;

/// Represents a connection to a database
pub trait Connection {
    fn create_statement(&self, sql: &str) -> Result<Rc<dyn Statement>>;
}

/// Represents a statement
pub trait Statement {
    /// Execute a query that is expected to return a result set, such as a `SELECT` statement
    fn execute_query(&self) -> Result<Rc<RefCell<dyn ResultSet>>>;
    /// Execute a query that is expected to update some rows.
    fn execute_update(&self) -> Result<usize>;
}

/// Result set from executing a query against a statement
pub trait ResultSet {
    /// Move the cursor to the next available row if one exists and return true if it does
    fn next(&mut self) -> bool;
    /// Get the i32 value at column `i` (1-based)
    fn get_i32(&self, i: usize) -> i32;
    /// Get the String value at column `i` (1-based)
    fn get_string(&self, i: usize) -> String;
    //TODO add accessors for all data types
}

