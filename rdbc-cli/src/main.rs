use std::cell::RefCell;
use std::rc::Rc;

use rustyline::Editor;

use rdbc::{Connection, Result};
use rdbc_mysql::MySQLDriver;
use rdbc_postgres::PostgresDriver;


//TODO: turn this into a CLI ... for now, just demonstrate that the same code can be used
// with Postgres and MySQL
fn main() -> Result<()> {
    let conn = connect_postgres()?;

    let mut rl = Editor::<()>::new();
    rl.load_history(".history").ok();

    let mut query = "".to_owned();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(ref line) if line.trim_end().ends_with(';') => {
                query.push_str(line.trim_end());
                rl.add_history_entry(query.clone());

                execute(conn.clone(), &query).unwrap();

                query = "".to_owned();
            }
            Ok(ref line) => {
                query.push_str(line);
                query.push_str(" ");
            }
            Err(_) => {
                break;
            }
        }
    }

    rl.save_history(".history").ok();

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
    let stmt = conn.prepare(sql)?;
    let mut stmt = stmt.borrow_mut();
    let rs = stmt.execute_query(&vec![])?;
    let mut rs = rs.borrow_mut();
    let meta = rs.meta_data().unwrap();

    for i in 0..meta.num_columns() {
        print!("{}\t", meta.column_name(i + 1));
    }
    println!();

    while rs.next() {
        for i in 0..meta.num_columns() {
            if i > 0 {
                print!("\t")
            }
            match meta.column_type(i + 1) {
                _ => print!("{:?}\t", rs.get_i32(i + 1)),
            }
        }
        println!();
    }

    Ok(())
}
