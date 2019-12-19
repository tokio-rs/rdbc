use std::cell::RefCell;
use std::rc::Rc;

use rdbc::{Connection, Result};
use rdbc_mysql::MySQLDriver;
use rdbc_postgres::PostgresDriver;
use std::collections::HashMap;

//TODO: turn this into a CLI ... for now, just demonstrate that the same code can be used
// with Postgres and MySQL
fn main() -> Result<()> {
    let conn = connect_postgres()?;
    execute(conn, "SELECT 1")?;

    let conn = connect_mysql()?;
    execute(conn, "SELECT 1")?;

    Ok(())
}

fn connect_mysql() -> Result<Rc<RefCell<dyn Connection + 'static>>> {
    let driver = MySQLDriver::new();
    driver.connect("mysql://root:secret@127.0.0.1:3307")
}

fn connect_postgres() -> Result<Rc<RefCell<dyn Connection>>> {
    let driver = PostgresDriver::new();
    driver.connect("postgres://rdbc:secret@127.0.0.1:5433")
}

fn execute(conn: Rc<RefCell<dyn Connection>>, sql: &str) -> Result<()> {
    println!("Executing {}", sql);
    let mut conn = conn.borrow_mut();
    let rs = conn.execute_query(sql, HashMap::new())?;
    let mut rs = rs.borrow_mut();
    while rs.next() {
        println!("{:?}", rs.get_i32(1))
    }
    Ok(())
}
