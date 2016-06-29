mod mysql;
mod connector;

#[cfg(test)]
mod tests {
    use super::connector::*;
    
    #[derive(Debug)]
    struct Post {
        id: u64,
        a: String,
        b: String,
        c: String
    }

    impl Storable for Post {
        fn store(fields: &Fields, row: Row) -> Self {
            panic!("TODO");
        }
    }

    #[test]
    fn it_works() {
        let mut mysql = Connector::new();
        mysql.connect("root:password@localhost/test").unwrap();

        let results: Vec<Post> = mysql.query("SELECT *, 1 as e FROM test.f").unwrap();

        println!("Results: {:?}", results);
    }
}
