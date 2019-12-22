use std::cell::RefCell;
use std::rc::Rc;

use odbc;
use odbc::odbc_safe::{AutocommitMode, Version, Odbc3};
use odbc::{Environment, HasResult, Allocated, NoResult, Prepared};

use rdbc;
use rdbc::{Error, ResultSet, Statement, Value, ResultSetMetaData};
use odbc::ResultSetState::{NoData, Data};

struct OdbcDriver {
    env: Environment<Odbc3>
}

impl OdbcDriver {

    pub fn new() -> Self {

        let env = odbc::create_environment_v3()
            .map_err(|e| e.unwrap())
            .unwrap();

        OdbcDriver { env }
    }

    pub fn connect(&self, connect_string: &str) -> Rc<RefCell<dyn rdbc::Connection + '_>> {
        let conn = self.env.connect_with_connection_string(connect_string).unwrap();
        Rc::new(RefCell::new(OdbcConnection { conn }))
    }
}

struct OdbcConnection<'a, V> where V: AutocommitMode {
    conn: odbc::Connection<'a, V>
}

impl<'a, V> rdbc::Connection for OdbcConnection<'a, V> where V: AutocommitMode {

    fn create(&mut self, sql: &str) -> rdbc::Result<Rc<RefCell<dyn rdbc::Statement + '_>>> {
        self.prepare(sql)
    }

    fn prepare(&mut self, sql: &str) -> rdbc::Result<Rc<RefCell<dyn rdbc::Statement + '_>>> {
        let stmt = odbc::Statement::with_parent(&self.conn).unwrap();
        let stmt = stmt.prepare(&sql).unwrap();
        Ok(Rc::new(RefCell::new(OdbcStatement { stmt })) as Rc<RefCell<dyn rdbc::Statement>>)
    }
}

struct OdbcStatement<'con, 'b, AC> where AC: AutocommitMode {
    stmt: odbc::Statement<'con, 'b, Prepared, NoResult, AC>,
}

impl<'con, 'b, AC> rdbc::Statement for OdbcStatement<'con, 'b, AC> where AC: AutocommitMode {

    fn execute_query(
        &mut self,
        params: &[Value],
    ) -> rdbc::Result<Rc<RefCell<dyn rdbc::ResultSet + '_>>> {

        //TODO bind params
        //self.stmt.bind_parameter(0, "foo");

//        match self.stmt.execute().unwrap() {
//            Data(mut stmt) => {
//                //Ok(Rc::new(RefCell::new(OdbcStatement { stmt, sql: sql.to_owned() })) as Rc<RefCell<dyn rdbc::Statement>>)
//                unimplemented!()
//            },
//            NoData(_) => unimplemented!()
//        }

        unimplemented!()
    }

    fn execute_update(&mut self, params: &[Value]) -> rdbc::Result<u64> {
        unimplemented!()
    }
}

struct OdbcResultSet {}

impl rdbc::ResultSet for OdbcResultSet {

    fn meta_data(&self) -> Result<Rc<ResultSetMetaData>, Error> {
        unimplemented!()
    }

    fn next(&mut self) -> bool {
        unimplemented!()
    }

    fn get_i32(&self, i: u64) -> Option<i32> {
        unimplemented!()
    }

    fn get_string(&self, i: u64) -> Option<String> {
        unimplemented!()
    }
}

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
