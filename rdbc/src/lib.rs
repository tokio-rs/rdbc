use std::rc::Rc;
use std::cell::RefCell;

pub type Result<T> = std::result::Result<T, String>;

pub trait Connection {
    fn create_statement(&self, sql: &str) -> Result<Rc<dyn Statement>>;
}

pub trait Statement {
    fn execute_query(&self) -> Result<Rc<RefCell<dyn ResultSet>>>;
    fn execute_update(&self) -> Result<usize>;
}

pub trait ResultSet {
    fn next(&mut self) -> bool;
    fn get_i32(&self, i: usize) -> i32;
    fn get_string(&self, i: usize) -> String;
    //TODO add accessors for all data types
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
