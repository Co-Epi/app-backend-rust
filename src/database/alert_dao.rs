use super::database::Database;
use crate::{
    errors::ServicesError,
    expect_log,
    reporting::{
        public_symptoms::{CoughSeverity, FeverSeverity, PublicSymptoms},
        symptom_inputs::UserInput,
    },
    reports_interval,
    reports_update::reports_updater::Alert,
};
use log::*;
use reports_interval::UnixTime;
use rusqlite::{params, Row, NO_PARAMS};
use std::sync::Arc;

pub trait AlertDao {
    fn all(&self) -> Result<Vec<Alert>, ServicesError>;
    fn save(&self, alerts: Vec<Alert>) -> Result<(), ServicesError>;
    fn delete(&self, id: String) -> Result<(), ServicesError>;
    fn update_is_read(&self, id: String, is_read: bool) -> Result<(), ServicesError>;
}

pub struct AlertDaoImpl {
    db: Arc<Database>,
}

impl AlertDaoImpl {
    pub fn new(db: Arc<Database>) -> AlertDaoImpl {
        Self::create_table_if_not_exists(&db);
        AlertDaoImpl { db }
    }

    fn create_table_if_not_exists(db: &Arc<Database>) {
        // TODO use blob for tcn? https://docs.rs/rusqlite/0.23.1/rusqlite/blob/index.html
        // TODO ideally FFI should send byte arrays too
        let res = db.execute_sql(
            "create table if not exists alert(
                id text primary key,
                start integer not null,
                end integer not null,
                min_distance real not null,
                avg_distance real not null,
                report_time integer not null,
                earliest_symptom_time integer,
                fever_severity integer not null,
                cough_severity integer not null,
                breathlessness integer not null,
                muscle_aches integer not null,
                loss_smell_or_taste integer not null,
                diarrhea integer not null,
                runny_nose integer not null,
                other integer not null,
                no_symptoms integer not null,
                report_id text not null,
                read integer not null,
                deleted integer
            )",
            params![],
        );
        expect_log!(res, "Couldn't create Alert table");
    }

    fn to_alert(row: &Row) -> Alert {
        let id_res = row.get(0);
        let id = expect_log!(id_res, "Invalid row: no id");

        let start_res = row.get(1);
        let start: i64 = expect_log!(start_res, "Invalid row: no start");

        let end_res = row.get(2);
        let end: i64 = expect_log!(end_res, "Invalid row: no end");

        let min_distance_res = row.get(3);
        let min_distance: f64 = expect_log!(min_distance_res, "Invalid row: no min_distance");

        let avg_distance_res = row.get(4);
        let avg_distance: f64 = expect_log!(avg_distance_res, "Invalid row: no avg_distance");

        let report_time_res = row.get(5);
        let report_time: i64 = expect_log!(report_time_res, "Invalid row: no report_time");

        let earliest_symptom_time_res = row.get(6);
        // TODO does this work for Option?
        let earliest_symptom_time: Option<i64> = expect_log!(
            earliest_symptom_time_res,
            "Invalid row: no earliest_symptom_time"
        );
        let earliest_symptom_time_unix_time: Option<UnixTime> =
            earliest_symptom_time.map(|t| UnixTime { value: t as u64 });

        let fever_severity_raw_res = row.get(7);
        let fever_severity_raw: i8 =
            expect_log!(fever_severity_raw_res, "Invalid row: no fever_severity");
        let fever_severity_res = FeverSeverity::from(fever_severity_raw as u8);
        let fever_severity = expect_log!(fever_severity_res, "Invalid raw value");

        let cough_severity_raw_res = row.get(8);
        let cough_severity_raw: i8 =
            expect_log!(cough_severity_raw_res, "Invalid row: no cough_severity");
        let cough_severity_res = CoughSeverity::from(cough_severity_raw as u8);
        let cough_severity = expect_log!(cough_severity_res, "Invalid raw value");

        let breathlessness_res = row.get(9);
        let breathlessness: i8 = expect_log!(breathlessness_res, "Invalid row: no breathlessness");

        let muscle_aches_res = row.get(10);
        let muscle_aches: i8 = expect_log!(muscle_aches_res, "Invalid row: no muscle_aches");

        let loss_smell_or_taste_res = row.get(11);
        let loss_smell_or_taste: i8 = expect_log!(
            loss_smell_or_taste_res,
            "Invalid row: no loss_smell_or_taste"
        );

        let diarrhea_res = row.get(12);
        let diarrhea: i8 = expect_log!(diarrhea_res, "Invalid row: no diarrhea");

        let runny_nose_res = row.get(13);
        let runny_nose: i8 = expect_log!(runny_nose_res, "Invalid row: no runny_nose");

        let other_res = row.get(14);
        let other: i8 = expect_log!(other_res, "Invalid row: no other");

        let no_symptoms_res = row.get(15);
        let no_symptoms: i8 = expect_log!(no_symptoms_res, "Invalid row: no no_symptoms");

        let report_id_res = row.get(16);
        let report_id = expect_log!(report_id_res, "Invalid row: no report_id");

        let read_res = row.get(17);
        let read: i8 = expect_log!(read_res, "Invalid row: no read");

        Alert {
            id,
            report_id,
            symptoms: PublicSymptoms {
                report_time: UnixTime {
                    value: report_time as u64,
                },
                earliest_symptom_time: UserInput::from(earliest_symptom_time_unix_time),
                fever_severity,
                cough_severity,
                breathlessness: to_bool(breathlessness),
                muscle_aches: to_bool(muscle_aches),
                loss_smell_or_taste: to_bool(loss_smell_or_taste),
                diarrhea: to_bool(diarrhea),
                runny_nose: to_bool(runny_nose),
                other: to_bool(other),
                no_symptoms: to_bool(no_symptoms),
            },
            contact_start: start as u64,
            contact_end: end as u64,
            min_distance: min_distance as f32,
            avg_distance: avg_distance as f32,
            is_read: to_bool(read),
        }
    }
}

impl AlertDao for AlertDaoImpl {
    fn all(&self) -> Result<Vec<Alert>, ServicesError> {
        self.db
            .query(
                "select 
                id,
                start,
                end,
                min_distance ,
                avg_distance,
                report_time,
                earliest_symptom_time,
                fever_severity,
                cough_severity,
                breathlessness,
                muscle_aches,
                loss_smell_or_taste,
                diarrhea,
                runny_nose,
                other,
                no_symptoms,
                report_id,
                read
                from alert where deleted is null",
                NO_PARAMS,
                |row| Self::to_alert(row),
            )
            .map_err(ServicesError::from)
    }

    fn delete(&self, id: String) -> Result<(), ServicesError> {
        debug!("Deleting alert with id: {}", id);

        let delete_res = self
            .db
            .execute_sql("update alert set deleted=1 where id=?;", params![id]);

        match delete_res {
            Ok(count) => {
                if count > 0 {
                    debug!("Updated: {} rows", count);
                    Ok(())
                } else {
                    error!("Didn't find alert to delete: {}", id);
                    Err(ServicesError::NotFound)
                }
            }
            Err(e) => Err(ServicesError::General(format!(
                "Error deleting alert: {}",
                e
            ))),
        }
    }

    fn update_is_read(&self, id: String, is_read: bool) -> Result<(), ServicesError> {
        debug!("Marking alert as read with id: {}", id);

        let delete_res = self.db.execute_sql(
            "update alert set read=? where id=?;",
            params![to_db_int(is_read), id],
        );

        match delete_res {
            Ok(count) => {
                if count > 0 {
                    debug!("Updated: {} rows", count);
                    Ok(())
                } else {
                    error!("Didn't find alert to mark as read: {}", id);
                    Err(ServicesError::NotFound)
                }
            }
            Err(e) => Err(ServicesError::General(format!(
                "Error marking alert as read: {}",
                e
            ))),
        }
    }

    fn save(&self, alerts: Vec<Alert>) -> Result<(), ServicesError> {
        self.db.transaction(|t| {
            for alert in alerts {
                t.execute(
                    "insert or ignore into alert(
                        id,
                        start,
                        end,
                        min_distance,
                        avg_distance,
                        report_time,
                        earliest_symptom_time,
                        fever_severity,
                        cough_severity,
                        breathlessness,
                        muscle_aches,
                        loss_smell_or_taste,
                        diarrhea,
                        runny_nose,
                        other,
                        no_symptoms,
                        report_id,
                        read
                    ) values(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                    params![
                        alert.id,
                        alert.contact_start as i64,
                        alert.contact_end as i64,
                        alert.min_distance as f64,
                        alert.avg_distance as f64,
                        alert.symptoms.report_time.value as i64,
                        alert
                            .symptoms
                            .earliest_symptom_time
                            .as_opt()
                            .map(|unix_time| unix_time.value as i64),
                        alert.symptoms.fever_severity.raw_value() as i64,
                        alert.symptoms.cough_severity.raw_value() as i64,
                        to_db_int(alert.symptoms.breathlessness),
                        to_db_int(alert.symptoms.muscle_aches),
                        to_db_int(alert.symptoms.loss_smell_or_taste),
                        to_db_int(alert.symptoms.diarrhea),
                        to_db_int(alert.symptoms.runny_nose),
                        to_db_int(alert.symptoms.other),
                        to_db_int(alert.symptoms.no_symptoms),
                        alert.report_id,
                        to_db_int(alert.is_read)
                    ],
                )?;
            }
            Ok(())
        })
    }
}

fn to_bool(db_int: i8) -> bool {
    if db_int == 1 {
        true
    } else if db_int == 0 {
        false
    } else {
        error!("Invalid db_int: {}", db_int);
        panic!()
    }
}

fn to_db_int(b: bool) -> i8 {
    if b {
        1
    } else {
        0
    }
}

// fn to_db_user_input(input: UserInput<T>) {

// }

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_saves_and_loads_alert() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let alert_dao = AlertDaoImpl::new(database.clone());

        let symptoms = PublicSymptoms {
            report_time: UnixTime { value: 0 },
            earliest_symptom_time: UserInput::Some(UnixTime { value: 1590356601 }),
            fever_severity: FeverSeverity::Mild,
            cough_severity: CoughSeverity::Dry,
            breathlessness: true,
            muscle_aches: true,
            loss_smell_or_taste: false,
            diarrhea: false,
            runny_nose: true,
            other: false,
            no_symptoms: true,
        };

        let alert = Alert {
            id: "1".to_owned(),
            report_id: "1".to_owned(),
            symptoms,
            contact_start: 1000,
            contact_end: 2000,
            min_distance: 2.3,
            avg_distance: 4.3,
            is_read: false,
        };

        let save_res = alert_dao.save(vec![alert.clone()]);
        assert!(save_res.is_ok());

        let loaded_alerts_res = alert_dao.all();
        assert!(loaded_alerts_res.is_ok());

        let loaded_alerts = loaded_alerts_res.unwrap();

        assert_eq!(loaded_alerts.len(), 1);
        assert_eq!(loaded_alerts[0], alert);
    }

    #[test]
    fn test_new_alert_with_same_id_ignored() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let alert_dao = AlertDaoImpl::new(database.clone());

        let symptoms = PublicSymptoms {
            report_time: UnixTime { value: 0 },
            earliest_symptom_time: UserInput::Some(UnixTime { value: 1590356601 }),
            fever_severity: FeverSeverity::Mild,
            cough_severity: CoughSeverity::Dry,
            breathlessness: true,
            muscle_aches: true,
            loss_smell_or_taste: false,
            diarrhea: false,
            runny_nose: true,
            other: false,
            no_symptoms: true,
        };

        let alert1 = Alert {
            id: "1".to_owned(),
            report_id: "1".to_owned(),
            symptoms: symptoms.clone(),
            contact_start: 1000,
            contact_end: 2000,
            min_distance: 2.3,
            avg_distance: 4.3,
            is_read: false,
        };

        let alert2 = Alert {
            id: "1".to_owned(),
            report_id: "1".to_owned(),
            symptoms: symptoms.clone(),
            contact_start: 1001,
            contact_end: 2001,
            min_distance: 2.4,
            avg_distance: 4.4,
            is_read: false,
        };

        let save_res = alert_dao.save(vec![alert1.clone(), alert2.clone()]);
        assert!(save_res.is_ok());

        let loaded_alerts_res = alert_dao.all();
        assert!(loaded_alerts_res.is_ok());

        let loaded_alerts = loaded_alerts_res.unwrap();

        assert_eq!(loaded_alerts.len(), 1);
        assert_eq!(loaded_alerts[0], alert1);
    }

    #[test]
    fn test_saves_and_loads_multiple_alerts() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let alert_dao = AlertDaoImpl::new(database.clone());

        let symptoms = PublicSymptoms {
            report_time: UnixTime { value: 0 },
            earliest_symptom_time: UserInput::Some(UnixTime { value: 1590356601 }),
            fever_severity: FeverSeverity::Mild,
            cough_severity: CoughSeverity::Dry,
            breathlessness: true,
            muscle_aches: true,
            loss_smell_or_taste: false,
            diarrhea: false,
            runny_nose: true,
            other: false,
            no_symptoms: true,
        };

        let alert1 = Alert {
            id: "1".to_owned(),
            report_id: "1".to_owned(),
            symptoms: symptoms.clone(),
            contact_start: 1000,
            contact_end: 2000,
            min_distance: 2.3,
            avg_distance: 4.3,
            is_read: false,
        };

        let alert2 = Alert {
            id: "2".to_owned(),
            report_id: "1".to_owned(),
            symptoms: symptoms.clone(),
            contact_start: 1001,
            contact_end: 2001,
            min_distance: 2.4,
            avg_distance: 4.4,
            is_read: true,
        };

        let save_res = alert_dao.save(vec![alert1.clone(), alert2.clone()]);
        assert!(save_res.is_ok());

        let loaded_alerts_res = alert_dao.all();
        assert!(loaded_alerts_res.is_ok());

        let loaded_alerts = loaded_alerts_res.unwrap();

        assert_eq!(loaded_alerts.len(), 2);
        assert_eq!(loaded_alerts[0], alert1);
        assert_eq!(loaded_alerts[1], alert2);
    }

    #[test]
    fn test_deletes_alert() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let alert_dao = AlertDaoImpl::new(database.clone());

        let symptoms = PublicSymptoms {
            report_time: UnixTime { value: 0 },
            earliest_symptom_time: UserInput::Some(UnixTime { value: 1590356601 }),
            fever_severity: FeverSeverity::Mild,
            cough_severity: CoughSeverity::Dry,
            breathlessness: true,
            muscle_aches: true,
            loss_smell_or_taste: false,
            diarrhea: false,
            runny_nose: true,
            other: false,
            no_symptoms: true,
        };

        let alert1 = Alert {
            id: "1".to_owned(),
            report_id: "1".to_owned(),
            symptoms: symptoms.clone(),
            contact_start: 1000,
            contact_end: 2000,
            min_distance: 2.3,
            avg_distance: 4.3,
            is_read: false,
        };

        let alert2 = Alert {
            id: "2".to_owned(),
            report_id: "1".to_owned(),
            symptoms: symptoms.clone(),
            contact_start: 1001,
            contact_end: 2001,
            min_distance: 2.4,
            avg_distance: 4.4,
            is_read: true,
        };

        let save_res = alert_dao.save(vec![alert1.clone(), alert2.clone()]);
        assert!(save_res.is_ok());

        let delete_res = alert_dao.delete("2".to_owned());
        assert!(delete_res.is_ok());

        let loaded_alerts_res = alert_dao.all();
        assert!(loaded_alerts_res.is_ok());

        let loaded_alerts = loaded_alerts_res.unwrap();

        assert_eq!(loaded_alerts.len(), 1);
        assert_eq!(loaded_alerts[0], alert1);
    }

    #[test]
    fn test_alert_not_restored_after_delete() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let alert_dao = AlertDaoImpl::new(database.clone());

        let symptoms = PublicSymptoms {
            report_time: UnixTime { value: 0 },
            earliest_symptom_time: UserInput::Some(UnixTime { value: 1590356601 }),
            fever_severity: FeverSeverity::Mild,
            cough_severity: CoughSeverity::Dry,
            breathlessness: true,
            muscle_aches: true,
            loss_smell_or_taste: false,
            diarrhea: false,
            runny_nose: true,
            other: false,
            no_symptoms: true,
        };

        let alert1 = Alert {
            id: "1".to_owned(),
            report_id: "1".to_owned(),
            symptoms: symptoms.clone(),
            contact_start: 1000,
            contact_end: 2000,
            min_distance: 2.3,
            avg_distance: 4.3,
            is_read: false,
        };

        let alert2 = Alert {
            id: "2".to_owned(),
            report_id: "1".to_owned(),
            symptoms: symptoms.clone(),
            contact_start: 1001,
            contact_end: 2001,
            min_distance: 2.4,
            avg_distance: 4.4,
            is_read: true,
        };

        let save_res = alert_dao.save(vec![alert1.clone(), alert2.clone()]);
        assert!(save_res.is_ok());

        let delete_res = alert_dao.delete("2".to_owned());
        assert!(delete_res.is_ok());

        let save_res = alert_dao.save(vec![alert2.clone()]);
        assert!(save_res.is_ok());

        let loaded_alerts_res = alert_dao.all();
        assert!(loaded_alerts_res.is_ok());

        let loaded_alerts = loaded_alerts_res.unwrap();

        assert_eq!(loaded_alerts.len(), 1);
        assert_eq!(loaded_alerts[0], alert1);
    }

    #[test]
    fn test_marks_alert_as_read() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let alert_dao = AlertDaoImpl::new(database.clone());

        let symptoms = PublicSymptoms {
            report_time: UnixTime { value: 0 },
            earliest_symptom_time: UserInput::Some(UnixTime { value: 1590356601 }),
            fever_severity: FeverSeverity::Mild,
            cough_severity: CoughSeverity::Dry,
            breathlessness: true,
            muscle_aches: true,
            loss_smell_or_taste: false,
            diarrhea: false,
            runny_nose: true,
            other: false,
            no_symptoms: true,
        };

        let alert = Alert {
            id: "1".to_owned(),
            report_id: "1".to_owned(),
            symptoms: symptoms.clone(),
            contact_start: 1000,
            contact_end: 2000,
            min_distance: 2.3,
            avg_distance: 4.3,
            is_read: false,
        };

        let save_res = alert_dao.save(vec![alert.clone()]);
        assert!(save_res.is_ok());

        let update_res = alert_dao.update_is_read("1".to_owned(), true);
        assert!(update_res.is_ok());

        let loaded_alerts_res = alert_dao.all();
        assert!(loaded_alerts_res.is_ok());

        let loaded_alerts = loaded_alerts_res.unwrap();

        assert_eq!(loaded_alerts.len(), 1);
        assert_eq!(
            loaded_alerts[0],
            Alert {
                is_read: true,
                ..alert
            }
        );
    }

    #[test]
    fn test_marks_alert_as_unread() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let alert_dao = AlertDaoImpl::new(database.clone());

        let symptoms = PublicSymptoms {
            report_time: UnixTime { value: 0 },
            earliest_symptom_time: UserInput::Some(UnixTime { value: 1590356601 }),
            fever_severity: FeverSeverity::Mild,
            cough_severity: CoughSeverity::Dry,
            breathlessness: true,
            muscle_aches: true,
            loss_smell_or_taste: false,
            diarrhea: false,
            runny_nose: true,
            other: false,
            no_symptoms: true,
        };

        let alert = Alert {
            id: "1".to_owned(),
            report_id: "1".to_owned(),
            symptoms: symptoms.clone(),
            contact_start: 1000,
            contact_end: 2000,
            min_distance: 2.3,
            avg_distance: 4.3,
            is_read: true,
        };

        let save_res = alert_dao.save(vec![alert.clone()]);
        assert!(save_res.is_ok());

        let update_res = alert_dao.update_is_read("1".to_owned(), false);
        assert!(update_res.is_ok());

        let loaded_alerts_res = alert_dao.all();
        assert!(loaded_alerts_res.is_ok());

        let loaded_alerts = loaded_alerts_res.unwrap();

        assert_eq!(loaded_alerts.len(), 1);
        assert_eq!(
            loaded_alerts[0],
            Alert {
                is_read: false,
                ..alert
            }
        );
    }

    #[test]
    fn test_marks_alert_as_read_if_already_read_does_nothing() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let alert_dao = AlertDaoImpl::new(database.clone());

        let symptoms = PublicSymptoms {
            report_time: UnixTime { value: 0 },
            earliest_symptom_time: UserInput::Some(UnixTime { value: 1590356601 }),
            fever_severity: FeverSeverity::Mild,
            cough_severity: CoughSeverity::Dry,
            breathlessness: true,
            muscle_aches: true,
            loss_smell_or_taste: false,
            diarrhea: false,
            runny_nose: true,
            other: false,
            no_symptoms: true,
        };

        let alert1 = Alert {
            id: "1".to_owned(),
            report_id: "1".to_owned(),
            symptoms: symptoms.clone(),
            contact_start: 1000,
            contact_end: 2000,
            min_distance: 2.3,
            avg_distance: 4.3,
            is_read: true,
        };

        let save_res = alert_dao.save(vec![alert1.clone()]);
        assert!(save_res.is_ok());

        let update_res = alert_dao.update_is_read("1".to_owned(), true);
        assert!(update_res.is_ok());

        let loaded_alerts_res = alert_dao.all();
        assert!(loaded_alerts_res.is_ok());

        let loaded_alerts = loaded_alerts_res.unwrap();

        assert_eq!(loaded_alerts.len(), 1);
        assert_eq!(
            loaded_alerts[0],
            Alert {
                is_read: true,
                ..alert1
            }
        );
    }
}
