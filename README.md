
# Rust DataBase Connectivity (RDBC)

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Docs](https://docs.rs/rdbc/badge.svg)](https://docs.rs/rdbc)
[![Version](https://img.shields.io/crates/v/rdbc.svg)](https://crates.io/crates/rdbc)

Love them or hate them, the [ODBC](https://en.wikipedia.org/wiki/Open_Database_Connectivity) and [JDBC](https://en.wikipedia.org/wiki/Java_Database_Connectivity) standards have made it easy to use a wide range of desktop and server products with many different databases thanks to the availability of database drivers implementing these standards.

I believe there is a need for a Rust equivalent so I have started this experimental project and aim to provide an RDBC API and reference implementations (drivers) for both Postgres and MySQL. 

It should then be easy for others to create new RDBC drivers for other databases.

# Why do we need this when we have Diesel?

This is filling a different need. I love the [Diesel](https://diesel.rs/) approach for building applications but if you are building a generic SQL tool, a business intelligence tool, or a distributed query engine, there is a need to connect to different databases and execute arbitrary SQL. This is where we need a standard API and available drivers.

# Connection Trait

Currently there is a simple `Connection` trait that allows queries to be executed that either return a `ResultSet` for reads or just the number of rows affected by writes. Later, there will be a `Driver` trait as well. 

```rust
/// Represents a connection to a database
pub trait Connection {
    /// Execute a query that is expected to return a result set, such as a `SELECT` statement
    fn execute_query(&mut self, sql: &str) -> Result<Rc<RefCell<dyn ResultSet + '_>>>;
    /// Execute a query that is expected to update some rows.
    fn execute_update(&mut self, sql: &str) -> Result<usize>;
}
```

# Examples

## Create a Postgres Connection

```rust
fn connect_postgres() -> Rc<RefCell<dyn Connection>> {
    let driver = PostgresDriver::new();
    driver.connect("postgres://rdbc:secret@127.0.0.1:5433")
}
```

## Create a MySQL Connection

```rust
fn connect_mysql() -> Rc<RefCell<dyn Connection>> {
    let driver = MySQLDriver::new();
    driver.connect("mysql://root:secret@127.0.0.1:3307").unwrap()
}
```

## Execute a Query

```rust
fn execute(conn: Rc<RefCell<dyn Connection>>, sql: &str) {
    println!("Execute {}", sql);
    let mut conn = conn.borrow_mut();
    let rs = conn.execute_query(sql).unwrap();
    let mut rs = rs.borrow_mut();
    while rs.next() {
        println!("{:?}", rs.get_i32(1))
    }
}
```

# Building

Use `docker-compose` to start up a Postgres and MySQL instance.

```bash
docker-compose up -d
```

Run `cargo test` to run the tests.