use super::mysql;
use std::ptr;
use std::ffi::CString;
use std::ffi::CStr;
use std::str;
use std::slice;

pub struct Connector {
    mysql: *mut mysql::st_mysql,
}

impl Connector {
    pub fn new() -> Self {
        let mysql = unsafe {
            let mut mysql = ptr::null_mut();
            mysql = mysql::mysql_init(mysql);

            if mysql.is_null() {
                panic!("oom or init problem for mysql connector");
            }

            mysql
        };
        Connector{
            mysql: mysql,
        }
    }

    pub fn connect<T: Into<String>>(&mut self, dsn: T) -> Result<(),Error> {
        let dsn = dsn.into();
        
        let mut iter = dsn.split("@");
        let user_and_pw = iter.next();
        let addr_and_db = iter.next();
        
        let mut iter = user_and_pw.unwrap().split(":");
        let username = iter.next();
        let password = iter.next();
    
        let mut iter = addr_and_db.unwrap().split("/");
        let addr = iter.next();
        let db = iter.next();

        let mut iter = addr.unwrap().split("/");
        let addr = iter.next();
        let port = iter.next();

        if user_and_pw.is_none() || addr_and_db.is_none() || username.is_none() || addr.is_none() || db.is_none() || addr.is_none() {
            return Err(Error::InvalidDSN);
        }

        /*
        println!("addr: {}, username: {}, password: {}, db: {}",
            addr.unwrap_or(""),
            username.unwrap_or(""),
            password.unwrap_or(""),
            db.unwrap_or("")
        );
        */

        let success = unsafe{
            mysql::mysql_real_connect(
                self.mysql,
                CString::new(addr.unwrap())         .unwrap().as_bytes_with_nul().as_ptr() as *const i8,
                CString::new(username.unwrap())     .unwrap().as_bytes_with_nul().as_ptr() as *const i8,
                CString::new(password.unwrap_or("")).unwrap().as_bytes_with_nul().as_ptr() as *const i8,
                CString::new(db.unwrap())           .unwrap().as_bytes_with_nul().as_ptr() as *const i8,
                port.map_or(Ok(0), |port| port.parse()).unwrap(),
                ptr::null_mut(),
                0,
            )
        };

        if success.is_null() {
            let err = get_error(self.mysql);
            return Err(Error::ConnectionFailure(err.into()));
        }

        Ok(())
    }

    pub fn query<X: TableMapper, T: Storable<X>>(&mut self, query: &'static str) -> Result<Vec<T>, Error>{
        println!("running query: {:?}", query);
        let c_query = CString::new(query).unwrap();
        unsafe{ mysql::mysql_query(self.mysql, c_query.as_ptr()) };
        
        let result = unsafe{ mysql::mysql_store_result(self.mysql)};
        if result.is_null() {
            let err = get_error(self.mysql);
            return Err(Error::QueryError(err.into()));
        };

        let mut rows = try!(Rows::new(self.mysql, result));
        println!("fields: {:?}", rows.fields);

        let mut results: Vec<T> = Vec::new();

        loop {
            let next = rows.next();
            let next = if next.is_some() {
                next.unwrap()
            } else {
                break;
            };

            let new = T::store(next);
            results.push(new);

        }

        Ok(results)
    }
}

impl Drop for Connector {
    fn drop(&mut self) {
        //println!("dropping {:?}", self.mysql);
        unsafe{ mysql::mysql_close(self.mysql) };
    }
}

#[derive(Debug, Clone)]
pub struct Fields {
    fields: Vec<mysql::st_mysql_field>,
    remaining: Vec<(usize, String)>,
}

impl Fields {
    fn new(fields: Vec<mysql::st_mysql_field>) -> Self {
        let mut ret = Fields{ fields: fields, remaining: vec![] };
        ret.remaining = ret.get_all_tables();
        ret
    }

    pub fn remove_remaining(&mut self) -> Vec<(usize, String)> {
        self.remaining.drain(0..).collect()
    }

    //pub fn get_remaining_until_next_table_name

    pub fn get_all_tables(&self) -> Vec<(usize, String)> {
        let mut res = vec![];
        for (i, row) in self.fields.iter().enumerate() {
            let table = unsafe {
                let table = CStr::from_ptr(row.table);
                let table = table.to_str();
                table.unwrap_or("").into()
            };
            res.push((i, table));
        }
        res
    }
}

/*
impl<A: Storable, B: Storable, T: TableMapper> Storable<T> for (A, B) {
    fn store(row: Row) -> Self {
        T::get_tables(row);
        panic!("TODO");
    }
}
*/

pub trait TableMapper {
    fn cols(row: &mut Row) -> Vec<(usize, String)>;
}

pub struct TMAll;

impl TableMapper for TMAll {
    fn cols(row: &mut Row) -> Vec<(usize, String)> {
        row.fields.remove_remaining()
    }
}

pub struct Rows {
    mysql: *mut mysql::st_mysql,
    res: *mut mysql::st_mysql_res,
    fields: Fields,
}

impl Rows {
    fn new(mysql: *mut mysql::st_mysql, res: *mut mysql::st_mysql_res) -> Result<Self, Error> {
        let fields = unsafe {
            let fields = mysql::mysql_fetch_fields(res);

            if fields.is_null() {
                let err = get_error(mysql);
                return Err(Error::FieldsError(err));
            }

            let fields = slice::from_raw_parts(
                fields as *const mysql::st_mysql_field,
                mysql::mysql_num_fields(res) as usize
            ).to_vec();

            Fields::new(fields)
        };

        Ok(Rows{ res: res, mysql: mysql, fields: fields })
    }
}

impl Drop for Rows {
    fn drop(&mut self) {
        unsafe{ mysql::mysql_free_result(self.res) };
    }
}

impl Iterator for Rows {
    type Item = Row;
    fn next(&mut self) -> Option<Self::Item> {
        let row = unsafe{ mysql::mysql_fetch_row(self.res) as mysql::MYSQL_ROW };
        if row.is_null() {
            return None;
        }

        Some(Row{
            row: row,
            fields: self.fields.clone() // TODO: fix this with some irritating lifetime stuff
        })
    }
}

#[derive(Debug)]
pub struct Row {
    row: mysql::MYSQL_ROW,
    //row: &[mysql::st_mysql_rows],
    pub fields: Fields,
}

impl Row {
    fn get_col_index(&self, cols: &mut Vec<(usize, String)>, col_name: &'static str) -> Option<usize> {
        let mut found: Option<usize> = None;
        for (i, tcol) in cols.iter().enumerate() {
            //println!("COMPARING {:?} == {:?}", tcol.1, col_name);

            let field = self.fields.fields[i]; // no bounds checking required
            //println!("field: {:?}", field);
            let field_name = unsafe{ CStr::from_ptr( field.name ) };
            
            /*
            let row = unsafe {
                slice::from_raw_parts(
                    self.row,
                    self.fields.fields.len() as usize
                ).to_vec()
            };
            
            println!("ROW: {:?}", row);

            for cell in row.into_iter() {
                let cstr = unsafe{ CStr::from_ptr(cell) };
                println!("CELL: {:?}", cstr);
            }
            */

            if field_name.to_string_lossy() == col_name {
                found = Some(i);
                break;
            }
        }
        println!("FOUND IS SOME: {:?}", found);
        if found.is_some() {
            let tcol = cols.swap_remove(found.unwrap());
            return Some(tcol.0);
        }

        None
    }

    pub fn get_u64(&self, cols: &mut Vec<(usize, String)>, col_name: &'static str) -> Option<u64> {
        self.get_col_index(cols, col_name).and_then(|index| {
            let cells = unsafe{ slice::from_raw_parts(self.row, self.fields.fields.len()) };
            let cell_text = unsafe{ CStr::from_ptr(cells[index]) }.to_string_lossy();
            println!("CELL: {:?}", cell_text);
            Some(cell_text.parse().unwrap())
        })
    }

    pub fn get_String(&self, cols: &mut Vec<(usize, String)>, col_name: &'static str) -> Option<String> {
        self.get_col_index(cols, col_name).and_then(|index| {
            let cells = unsafe{ slice::from_raw_parts(self.row, self.fields.fields.len()) };
            let cell_text = unsafe{ CStr::from_ptr(cells[index]) }.to_string_lossy();
            println!("CELL: {:?}", cell_text);
            Some(cell_text.into_owned().into())
        })
    }
}

pub trait Storable<T: TableMapper> {
    fn store(Row) -> Self;
}

#[derive(Debug)]
pub enum Error {
    InvalidDSN,
    Utf8Error(str::Utf8Error),
    ConnectionFailure(String),
    QueryError(String),
    FieldsError(String),
}

impl From<str::Utf8Error> for Error {
    fn from(src: str::Utf8Error) -> Self {
        Error::Utf8Error(src)
    }
}

fn get_error(mysql: *mut mysql::st_mysql) -> String {
    let err = unsafe{ mysql::mysql_error(mysql) };
    let err = unsafe{ CStr::from_ptr(err) };

    let err = err.to_str();

    err.unwrap_or("").into()
}
