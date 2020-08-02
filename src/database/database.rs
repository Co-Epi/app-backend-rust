use crate::{errors::ServicesError, expect_log};
use log::*;
use rusqlite::{Connection, Result};
use rusqlite::{Row, ToSql, Transaction};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    // Returns number of rows that were changed / inserted / deleted
    // Can be abstracted more in the future...
    pub fn execute_sql<P>(&self, sql: &str, pars: P) -> Result<usize, rusqlite::Error>
    where
        P: IntoIterator,
        P::Item: ToSql,
    {
        let res = self.conn.lock();
        let conn = expect_log!(res, "Couldn't lock mutex");
        conn.execute(sql, pars)
    }
    #[allow(dead_code)]
    pub fn execute_batch(&self, sql: &str) -> Result<()>
    {
        let res = self.conn.lock();
        let conn = expect_log!(res, "Couldn't lock mutex");
        conn.execute_batch(sql)
    }

    pub fn core_pragma_query(&self, pragma_variable_name: &str) -> i32 {
        let res = self.conn.lock();
        let conn = expect_log!(res, "Couldn't lock mutex");
        let mut value = 0;
        let _ = conn.pragma_query(None, pragma_variable_name, |row| {
                let value_res= row.get(0);
                value = value_res.unwrap();
                Ok(())
        });
        value
    }

    pub fn core_pragma_update(&self, pragma_variable_name: &str, new_value: &i32){
        let res = self.conn.lock();
        let conn = expect_log!(res, "Couldn't lock mutex");
        // let mut value = 0;

        // let value = conn.pragma_update_and_check(None, pragma_varible_name, new_value, |row|{row.get(0)}).unwrap();
        // value

        let _ = conn.pragma_update(None, pragma_variable_name, new_value);

    }

    pub fn query<T, P, F>(&self, sql: &str, params: P, f: F) -> Result<Vec<T>, rusqlite::Error>
    where
        P: IntoIterator,
        P::Item: ToSql,
        F: Fn(&Row<'_>) -> T,
    {
        let res = self.conn.lock();
        let conn = expect_log!(res, "Couldn't lock mutex");

        let mut statement = conn.prepare(sql)?;
        let mut rows = statement.query(params)?;

        let mut objs = Vec::new();
        while let Some(row) = rows.next().unwrap() {
            objs.push(f(row));
        }
        Ok(objs)
    }

    pub fn query_row<T, P, F>(&self, sql: &str, params: P, f: F) -> Result<T, rusqlite::Error>
    where
        P: IntoIterator,
        P::Item: ToSql,
        F: FnOnce(&Row<'_>) -> Result<T, rusqlite::Error>,
    {
        let res = self.conn.lock();
        let conn = expect_log!(res, "Couldn't lock mutex");
        conn.query_row(sql, params, f)
    }

    pub fn transaction<F>(&self, f: F) -> Result<(), ServicesError>
    where
        F: FnOnce(&Transaction) -> Result<(), ServicesError>,
    {
        let conn_res = self.conn.lock();
        let mut conn = expect_log!(conn_res, "Couldn't lock connection");

        let t = conn.transaction()?;
        match f(&t) {
            Ok(_) => t.commit().map_err(ServicesError::from),
            Err(commit_error) => {
                let rollback_res = t.rollback();
                if rollback_res.is_err() {
                    // As we're already returning error status, show only a log for rollback error.
                    error!(
                        "There was an error committing and rollback failed too with: {:?}",
                        rollback_res
                    );
                }
                Err(commit_error)
            }
        }
    }

    pub fn new(conn: Connection) -> Database {
        let load_array_mod_res = rusqlite::vtab::array::load_module(&conn);
        expect_log!(
            load_array_mod_res,
            "Couldn't load array module (needed for IN query)"
        );
        Database {
            conn: Mutex::new(conn),
        }
    }
}
