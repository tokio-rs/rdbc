pub type Result<T> = std::result::Result<T, String>;

pub trait Connection {
    fn create_statement(&self, sql: &str) -> Result<Box<dyn Statement>>;
}

pub trait Statement {
    fn execute_query(&self) -> Result<Box<dyn ResultSet>>;
    fn execute_update(&self) -> Result<usize>;
}

pub trait ResultSet {
    fn next(&self) -> bool;
    fn get_i32(&self, i: usize) -> i32;
    fn get_string(&self, i: usize) -> String;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
