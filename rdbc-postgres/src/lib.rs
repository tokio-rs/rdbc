use rdbc;

use postgres::{Connection, TlsMode};
use rdbc::ResultSet;

struct PConnection {
    conn: Box<Connection>
}

impl PConnection {
    pub fn new(conn: Connection) -> Self {
        Self { conn: Box::new(conn) }
    }
}

impl rdbc::Connection for PConnection {

    fn create_statement(&self, sql: &str) -> rdbc::Result<Box<dyn rdbc::Statement>> {
        unimplemented!()
    }

}

struct PStatement {
    conn: Box<Connection>,
    sql: String
}

impl rdbc::Statement for PStatement {

    fn execute_query(&self) -> rdbc::Result<Box<dyn ResultSet>> {
        let rs = self.conn.query(&self.sql, &[]).unwrap();
        unimplemented!()
    }

    fn execute_update(&self) -> rdbc::Result<usize> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rdbc::Connection;

    #[test]
    fn it_works() {
        let conn = postgres::Connection::connect("postgres://postgres@localhost:5433", TlsMode::None).unwrap();
        let conn: Box<Connection> = Box::new(PConnection::new(conn));

        let stmt = conn.create_statement("CREATE TABLE person (
                    id              SERIAL PRIMARY KEY,
                    name            VARCHAR NOT NULL,
                    data            BYTEA
                  )").unwrap();

        let rs = stmt.execute_update().unwrap();


//        let me = Person {
//            id: 0,
//            name: "Steven".to_string(),
//            data: None,
//        };
//        conn.execute("INSERT INTO person (name, data) VALUES ($1, $2)",
//                     &[&me.name, &me.data]).unwrap();
//        for row in &conn.query("SELECT id, name, data FROM person", &[]).unwrap() {
//            let person = Person {
//                id: row.get(0),
//                name: row.get(1),
//                data: row.get(2),
//            };
//            println!("Found person {}", person.name);
//        }
    }
}
