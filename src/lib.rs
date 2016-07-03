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

    impl<T: TableMapper> Storable<T> for Post {
        fn store(mut row: Row) -> Self {
            let mut cols = T::cols(&mut row);
            Post{
                id: row.get_u64   (&mut cols, "id"),
                a:  row.get_String(&mut cols, "a"),
                b:  row.get_String(&mut cols, "b"),
                c:  row.get_String(&mut cols, "c"),
                d:  row.get_u64   (&mut cols, "d"),
            }
        }
    }

    #[derive(Debug)]
    struct Reply {
        id: Option<u64>,
        f_id: Option<u64>,
        data: Option<String>,
    }

    impl<T: TableMapper> Storable<T> for Post {
        fn store(mut row: Row) -> Self {
            let mut cols = T::cols(&mut row);
            Post{
                id:   row.get_u64   (&mut cols, "id"),
                f_id: row.get_String(&mut cols, "f_id"),
                data: row.get_u64   (&mut cols, "data"),
            }
        }
    }

    #[test]
    fn it_works() {
        let mut mysql = Connector::new();
        mysql.connect("root:password@localhost/test").unwrap();

        let results: Vec<Post> = mysql.query::<TMAll, _>("SELECT *, 1 as d FROM test.f").unwrap();

        println!("Results: {:?}", results);
    }

    #[test]
    fn it_works_2() {
        let mut mysql = Connector::new();
        mysql.connect("root:password@localhost/test").unwrap();

        let query = "SELECT * FROM f LEFT JOIN g ON(g.f_id = f.id)";
        let results: Vec<(Post, Reply)> = mysql.query::<(TMAll, TMAll), _>(query).unwrap();

        println!("Results: {:?}", results);
    }
}
