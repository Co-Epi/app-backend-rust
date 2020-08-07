use crate::{errors::ServicesError, expect_log};
use log::*;
use rusqlite::types::FromSql;
use rusqlite::{Connection, Error, Result, Row, ToSql, Transaction};
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
    pub fn execute_batch(&self, sql: &str) -> Result<()> {
        let res = self.conn.lock();
        let conn = expect_log!(res, "Couldn't lock mutex");
        conn.execute_batch(sql)
    }

    pub fn core_pragma_query<T>(&self, pragma_variable_name: &str) -> T
    where
        T: FromSql,
    {
        let res = self.conn.lock();
        let conn = expect_log!(res, "Couldn't lock mutex");
        let mut value_res: Result<T, Error> = Err(Error::QueryReturnedNoRows);
        let _ = conn.pragma_query(None, pragma_variable_name, |row| {
            value_res = row.get(0);
            Ok(())
        });
        expect_log!(value_res, "Failed to retrieve pragma value")
    }

    pub fn core_pragma_update(&self, pragma_variable_name: &str, new_value: &i32) {
        let res = self.conn.lock();
        let conn = expect_log!(res, "Couldn't lock mutex");
        let update_res = conn.pragma_update(None, pragma_variable_name, new_value);
        expect_log!(update_res, "Failed to update pragma value");
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_pragma_logic() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let pragma_variable_name = "user_version";
        let db_version: i32 = database.core_pragma_query(pragma_variable_name);
        assert_eq!(0, db_version);
        database.core_pragma_update(pragma_variable_name, &17);
        let db_version_17: i32 = database.core_pragma_query(pragma_variable_name);
        assert_eq!(17, db_version_17);

        database.core_pragma_update(pragma_variable_name, &1024);
        let db_version_1024: i32 = database.core_pragma_query(pragma_variable_name);
        assert_eq!(1024, db_version_1024);
    }
}
