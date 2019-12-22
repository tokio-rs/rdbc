
# Rust DataBase Connectivity (RDBC)

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Docs](https://docs.rs/rdbc/badge.svg)](https://docs.rs/rdbc)
[![Version](https://img.shields.io/crates/v/rdbc.svg)](https://crates.io/crates/rdbc)

Love them or hate them, the [ODBC](https://en.wikipedia.org/wiki/Open_Database_Connectivity) and [JDBC](https://en.wikipedia.org/wiki/Java_Database_Connectivity) standards have made it easy to use a wide range of desktop and server products with many different databases thanks to the availability of database drivers implementing these standards.

I believe there is a need for a Rust equivalent so I have started this experimental project and aim to provide an RDBC API and reference implementations (drivers) for both Postgres and MySQL. 

Note that the provided RDBC drivers are just wrappers around the existing `postgres` and `mysql` crates and this project is not attempting to build new drivers from scratch but rather make it possible to leverage existing drivers through a common API.

# Why do we need this when we have Diesel?

This is filling a different need. I love the [Diesel](https://diesel.rs/) approach for building applications but if you are building a generic SQL tool, a business intelligence tool, or a distributed query engine, there is a need to connect to different databases and execute arbitrary SQL. This is where we need a standard API and available drivers.

# RDBC API

Currently there are traits representing `Connection`, `Statement`, and `ResultSet`. Later, there will be a `Driver` trait as well as traits for retrieving database and result set meta-data.

Note that the design is currently purposely not idiomatic Rust and is modeled after ODBC and JDBC (including those annoying 1-based indices for looking up values). These traits can be wrapped by idiomatic Rust code and there will be features added to RDBC to facilitate that.

```rust
/// Represents a connection to a database
pub trait Connection {
    /// Prepare a SQL statement for execution
    fn prepare(&mut self, sql: &str) -> Result<Rc<RefCell<dyn Statement + '_>>>;
}

pub trait Statement {
    /// Execute a query that is expected to return a result set, such as a `SELECT` statement
    fn execute_query(&mut self, params: &Vec<Value>) -> Result<Rc<RefCell<dyn ResultSet + '_>>>;

    /// Execute a query that is expected to update some rows.
    fn execute_update(&mut self, params: &Vec<Value>) -> Result<usize>;
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
```

# Examples

## Create a Postgres Connection

```rust
fn connect_postgres() -> Result<Rc<RefCell<dyn Connection>>> {
    let driver = PostgresDriver::new();
    driver.connect("postgres://rdbc:secret@127.0.0.1:5433")
}
```

## Create a MySQL Connection

```rust
fn connect_mysql() -> Result<Rc<RefCell<dyn Connection>>> {
    let driver = MySQLDriver::new();
    driver.connect("mysql://root:secret@127.0.0.1:3307")
}
```

## Execute a Query

```rust
let conn = connect_postgres()?;
let mut conn = conn.borrow_mut();
let stmt = conn.prepare("SELECT a FROM b WHERE c = ?")?;
let mut stmt = stmt.borrow_mut();
let rs = stmt.execute_query(&vec![Value::Int32(123)])?;
let mut rs = rs.borrow_mut();
while rs.next() {
  println!("{:?}", rs.get_string(1));
}
```

# Current Status

This is just an experimental PoC and is not currently suitable for anything. However, I do intend to make it useful pretty quickly and I am tracking issues [here](https://github.com/andygrove/rdbc/issues).

The immediate priorities though are:

- [x] Announce project and get initial feedback
- [x] Support parameterized queries using positional parameters and prepared statements
- [ ] Support parameterized queries using positional parameters and non-prepared statements
- [ ] Implement unit and integration tests

# Building

Use `docker-compose up -d` to start up Postgres and MySQL containers to test against.

Use `cargo test` to run the unit tests.
