
# Rust DataBase Connectivity (RDBC)

Love them or hate them, the [ODBC](https://en.wikipedia.org/wiki/Open_Database_Connectivity) and [JDBC](https://en.wikipedia.org/wiki/Java_Database_Connectivity) standards have made it easy to use a wide range of desktop and server products with many different databases thanks to the availability of database drivers implementing these standards.

I believe there is a need for a Rust equivalent so I have started this experimental project and aim to provide an RDBC API and reference implementations (drivers) for both Postgres and MySQL. 

It should then be easy for others to create new RDBC drivers for other databases.

# Why do we need this when we have Diesel?

This is filling a different need. I love the [Diesel](https://diesel.rs/) approach for building applications but if you are building a generic SQL tool, a business intelligence tool, or a distributed query engine, there is a need to connect to different databases and execute arbitrary SQL. This is where we need a standard API and available drivers.

# Code Sample

```rust

// postgres specific code
let conn = postgres::Connection::connect("postgres://postgres@localhost:5433", TlsMode::None).unwrap();
let conn: Rc<dyn Connection> = Rc::new(PConnection::new(conn));

// generic RDBC code
let stmt = conn.create_statement("SELECT foo FROM bar").unwrap();
let rs = stmt.execute_query().unwrap();
let mut rs = rs.borrow_mut();
while rs.next() {
    println!("{}", rs.get_string(1))
}
```

 
