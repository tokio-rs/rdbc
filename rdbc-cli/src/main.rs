use std::cell::RefCell;
use std::rc::Rc;

use clap::{crate_version, App, Arg};
use rustyline::Editor;

use rdbc::{Connection, DataType, Result};
use rdbc_mysql::MySQLDriver;
use rdbc_postgres::PostgresDriver;

fn main() -> Result<()> {
    let matches = App::new("rdbc-cli")
        .version(crate_version!())
        .about("Rust DataBase Connectivity CLI")
        .arg(
            Arg::with_name("driver")
                .help("RDBC driver name")
                .short("d")
                .long("driver")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("connection-url")
                .help("The database connection URL")
                .short("c")
                .long("connection-url")
                .takes_value(true),
        )
        .get_matches();

    let driver = matches.value_of("driver").unwrap();
    let url = matches.value_of("connection-url").unwrap();
    println!("Connecting to {} driver with url: {}", driver, url);

    let driver = match driver {
        "mysql" => Rc::new(MySQLDriver::new()) as Rc<dyn rdbc::Driver>,
        "postgres" => Rc::new(PostgresDriver::new()) as Rc<dyn rdbc::Driver>,
        _ => panic!("Invalid driver"),
    };

    let conn = driver.connect(url).unwrap();

    let mut rl = Editor::<()>::new();
    rl.load_history(".history").ok();

    let mut query = "".to_owned();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(ref line) if line.trim_end().ends_with(';') => {
                query.push_str(line.trim_end());
                rl.add_history_entry(query.clone());

                match execute(conn.clone(), &query) {
                    Ok(_) => {}
                    Err(e) => println!("Error: {:?}", e),
                }

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

fn execute(conn: Rc<RefCell<dyn Connection>>, sql: &str) -> Result<()> {
    println!("Executing {}", sql);
    let mut conn = conn.borrow_mut();
    let stmt = conn.create(sql)?;
    let mut stmt = stmt.borrow_mut();
    let rs = stmt.execute_query(&vec![])?;
    let mut rs = rs.borrow_mut();
    let meta = rs.meta_data()?;

    for i in 0..meta.num_columns() {
        if i > 0 {
            print!("\t");
        }
        print!("{}", meta.column_name(i));
    }
    println!();

    while rs.next() {
        for i in 0..meta.num_columns() {
            if i > 0 {
                print!("\t");
            }
            match meta.column_type(i) {
                DataType::Utf8 => print!("{:?}", rs.get_string(i)),
                DataType::Integer => print!("{:?}", rs.get_i32(i)),
                // TODO other types
                _ => print!("{:?}", rs.get_string(i)),
            }
        }
        println!();
    }

    Ok(())
}
