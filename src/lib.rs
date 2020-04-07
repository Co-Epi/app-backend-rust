#![allow(warnings)]

use cen::*;
use once_cell::sync::{Lazy, OnceCell};
use persy::{Config, PRes, Persy, ValueMode};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Res<T> = Result<T, Error>;

const CENS_BY_TS: &'static str = "cens by ts";

pub fn init<P: AsRef<Path>>(p: P) -> PRes<()> {
    let db = Persy::open_or_create_with(p, Config::new(), |db| {
        let mut tx = db.begin()?;
        tx.create_index::<i64, u128>(CENS_BY_TS, ValueMode::CLUSTER)?;
        tx.prepare_commit()?.commit()?;
        Ok(())
    })?;
    DB.set(db);
    Ok(())
}

static DB: OnceCell<Persy> = OnceCell::new();
const DB_UNINIT: &'static str = "DB not initialized";

fn u128_of_cen(cen: ContactEventNumber) -> u128 {
    u128::from_le_bytes(cen.0)
}

fn cen_of_u128(u: u128) -> ContactEventNumber {
    ContactEventNumber(u.to_le_bytes())
}

fn cens_in_interval(start: i64, end: i64) -> Res<HashMap<u128, i64>> {
    let mut out = HashMap::new();

    let items = DB
        .get()
        .ok_or(DB_UNINIT)?
        .range::<i64, u128, _>(CENS_BY_TS, (start..end))?;

    for (ts, cens) in items {
        match cens {
            persy::Value::SINGLE(cen) => {
                out.insert(cen, ts);
            }
            persy::Value::CLUSTER(cens) => {
                for cen in cens {
                    out.insert(cen, ts);
                }
            }
        }
    }

    Ok(out)
}

pub fn record_cen(ts: i64, cen: ContactEventNumber) -> Res<()> {
    let db = DB.get().ok_or(DB_UNINIT)?;
    let mut tx = db.begin()?;
    tx.put(CENS_BY_TS, ts, u128_of_cen(cen))?;
    tx.prepare_commit()?.commit()?;
    Ok(())
}

pub fn delete_cens_between(start: i64, end: i64) -> Res<()> {
    let db = DB.get().ok_or(DB_UNINIT)?;
    let mut tx = db.begin()?;

    let tsv = tx
        .range::<i64, u128, _>(CENS_BY_TS, start..end)?
        .map(|(ts, _)| ts)
        .collect::<Vec<_>>();

    for ts in tsv {
        tx.remove::<i64, u128>(CENS_BY_TS, ts, None)?;
    }

    tx.prepare_commit()?.commit()?;
    Ok(())
}

pub fn match_cens_interval<'a, I: Iterator<Item = &'a Report>>(
    start: i64,
    end: i64,
    reports: I,
    // TODO: consider an output type that gives more info re severity - how many different times did we interact with this report?
) -> Res<Vec<(MemoType, Vec<u8>)>> {
    let in_interval = cens_in_interval(start, end)?;
    let mut out = Vec::new();

    for report in reports {
        for cen in report.contact_event_numbers() {
            if in_interval.contains_key(&u128_of_cen(cen)) {
                out.push((report.memo_type(), report.memo_data().to_vec()));
                break;
            }
        }
    }

    Ok(out)
}
