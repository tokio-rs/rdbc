
# Rust DataBase Connectivity (RDBC)

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Docs](https://docs.rs/rdbc/badge.svg)](https://docs.rs/rdbc)
[![Version](https://img.shields.io/crates/v/rdbc.svg)](https://crates.io/crates/rdbc)

Love them or hate them, the [ODBC](https://en.wikipedia.org/wiki/Open_Database_Connectivity) and [JDBC](https://en.wikipedia.org/wiki/Java_Database_Connectivity) standards have made it easy to use a wide range of desktop and server products with many different databases thanks to the availability of database drivers implementing these standards.

I believe there is a need for a Rust equivalent so I have started this experimental project and aim to provide an RDBC API and reference implementations (drivers) for both Postgres and MySQL. 

It should then be easy for others to create new RDBC drivers for other databases.

# Why do we need this when we have Diesel?

This is filling a different need. I love the [Diesel](https://diesel.rs/) approach for building applications but if you are building a generic SQL tool, a business intelligence tool, or a distributed query engine, there is a need to connect to different databases and execute arbitrary SQL. This is where we need a standard API and available drivers.

# Postgres Example

```rust

let driver = PostgresDriver::new();
let conn = driver.connect("postgres://postgres@localhost:5433");
let stmt = conn.create_statement("SELECT foo FROM bar").unwrap();
let rs = stmt.execute_query().unwrap();
let mut rs = rs.borrow_mut();
while rs.next() {
    println!("{}", rs.get_string(1))
}
```

# MySQL Example

```rust

let driver = MySQLDriver::new();
let conn = driver.connect("mysql://root:password@localhost:3307/mysql").unwrap();
let stmt = conn.create_statement("SELECT foo FROM bar").unwrap();
let rs = stmt.execute_query().unwrap();
let mut rs = rs.borrow_mut();
while rs.next() {
    println!("{}", rs.get_string(1))
}
```

# Building

```bash
docker-compose up -d
```

```bash
cargo test
``` 
