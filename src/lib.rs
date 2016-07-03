mod mysql;
mod connector;

#[cfg(test)]
mod tests {
    use super::connector::*;
    
    #[derive(Debug)]
    struct Post {
        id: Option<u64>,
        a: Option<String>,
        b: Option<String>,
        c: Option<String>,
        d: Option<u64>,
    }

    impl Storable for Post {
        type Kind = Post;
        fn store(mut row: Box<Row>) -> Self {
            Post{
                id: row.get_u64   ("id"),
                a:  row.get_string("a"),
                b:  row.get_string("b"),
                c:  row.get_string("c"),
                d:  row.get_u64   ("d"),
            }
        }
    }
    #[test]
    fn it_works() {
        let mut mysql = Connector::new();
        mysql.connect("root:password@localhost/test").unwrap();

        let results: Vec<_> = mysql.query::<Post>("SELECT *, 1 as d FROM test.f").unwrap();

        println!("Results: {:?}", results);
    }
}
