use std::cell::RefCell;
use std::rc::Rc;

use odbc;
use rdbc;
use rdbc::{Error, ResultSet, Statement, Value};

struct OdbcDriver {}

impl OdbcDriver {
    pub fn new() -> Self {
        OdbcDriver {}
    }

    pub fn connect(&self, connect_string: &str) -> Rc<RefCell<dyn rdbc::Connection + '_>> {
        let env = odbc::create_environment_v3()
            .map_err(|e| e.unwrap())
            .unwrap();
        let conn = env.connect_with_connection_string(connect_string).unwrap();
        unimplemented!()
        //Rc::new(RefCell::new(OdbcConnection { conn }))
    }
}

struct OdbcConnection {
    //    conn: odbc::Connection
}

impl rdbc::Connection for OdbcConnection {
    fn prepare(&mut self, sql: &str) -> rdbc::Result<Rc<RefCell<dyn rdbc::Statement + '_>>> {
        unimplemented!()
    }
}

struct OdbcStatement {}

impl rdbc::Statement for OdbcStatement {
    fn execute_query(
        &mut self,
        params: &Vec<Value>,
    ) -> rdbc::Result<Rc<RefCell<dyn rdbc::ResultSet + '_>>> {
        unimplemented!()
    }

    fn execute_update(&mut self, params: &Vec<Value>) -> rdbc::Result<usize> {
        unimplemented!()
    }
}

struct OdbcResultSet {}

impl rdbc::ResultSet for OdbcResultSet {
    fn next(&mut self) -> bool {
        unimplemented!()
    }

    fn get_i32(&self, i: usize) -> Option<i32> {
        unimplemented!()
    }

    fn get_string(&self, i: usize) -> Option<String> {
        unimplemented!()
    }
}

//
//fn connect() -> std::result::Result<(), DiagnosticRecord> {
//
//    let env = create_environment_v3().map_err(|e| e.unwrap())?;
//
//    let mut buffer = String::new();
//    println!("Please enter connection string: ");
//    io::stdin().read_line(&mut buffer).unwrap();
//
//    let conn = env.connect_with_connection_string(&buffer)?;
//    execute_statement(&conn)
//}
//
//fn execute_statement<'env>(conn: &Connection<'env>) -> Result<()> {
//    let stmt = Statement::with_parent(conn)?;
//
//    match stmt.exec_direct(&sql_text)? {
//        Data(mut stmt) => {
//            let cols = stmt.num_result_cols()?;
//            while let Some(mut cursor) = stmt.fetch()? {
//                for i in 1..(cols + 1) {
//                    match cursor.get_data::<&str>(i as u16)? {
//                        Some(val) => print!(" {}", val),
//                        None => print!(" NULL"),
//                    }
//                }
//                println!("");
//            }
//        }
//        NoData(_) => println!("Query executed, no data returned"),
//    }
//
//    Ok(())
//}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
