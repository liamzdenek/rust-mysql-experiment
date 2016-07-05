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
        fn store<T>(mut rows: T) -> Vec<Self> where T: Iterator<Item=Row>{
            rows.map(|mut row|{
                Post{
                    id: row.get_u64   ("id"),
                    a:  row.get_string("a"),
                    b:  row.get_string("b"),
                    c:  row.get_string("c"),
                    d:  row.get_u64   ("d"),
                }
            }).collect()
        }
    }

    #[derive(Debug)]
    struct Reply {
        id: Option<u64>,
        f_id: Option<u64>,
        data: Option<String>,
    }

    impl Storable for Reply {
        type Kind = Reply;
        fn store<T>(mut rows: T) -> Vec<Self> where T: Iterator<Item=Row>{
            rows.map(|mut row| {
                Reply{
                    id:   row.get_u64   ("id"),
                    f_id: row.get_u64   ("data"),
                    data: row.get_string("f_id"),
                }
            }).collect()
        }
    }

    #[test]
    fn test_simple() {
        let mut mysql = Connector::new();
        mysql.connect("root:password@127.0.0.1/test").unwrap();

        let results: Vec<_> = mysql.query::<Post>("SELECT *, 1 as d FROM test.f").unwrap();
        println!("Results: {:?}", results);
        
        let results: Vec<_> = mysql.query::<LeftJoin<Post, Reply, RSNextId>>("SELECT * FROM f LEFT JOIN g ON(f.id = g.f_id)").unwrap();
        println!("Results: {:?}", results);
    }
}
