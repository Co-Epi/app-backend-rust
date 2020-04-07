use cen::*;
use once_cell::sync::OnceCell;
use persy::{Config, Persy, ValueMode};
use std::{collections::HashMap, path::Path};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Res<T> = Result<T, Error>;

const CENS_BY_TS: &str = "cens by ts";

pub fn init<P: AsRef<Path>>(p: P) -> Res<()> {
    let db = Persy::open_or_create_with(p, Config::new(), |db| {
        let mut tx = db.begin()?;
        tx.create_index::<i64, u128>(CENS_BY_TS, ValueMode::CLUSTER)?;
        tx.prepare_commit()?.commit()?;
        Ok(())
    })?;
    DB.set(db).map_err(|_| DB_ALREADY_INIT)?;
    Ok(())
}

const DB_ALREADY_INIT: &str = "DB failed to initalize";

static DB: OnceCell<Persy> = OnceCell::new();
const DB_UNINIT: &str = "DB not initialized";

fn u128_of_cen(cen: ContactEventNumber) -> u128 {
    u128::from_le_bytes(cen.0)
}

// maybe we don't care about this one?
// leaving it here in case I need it as the library evolves
// TODO: consider deleting
// fn cen_of_u128(u: u128) -> ContactEventNumber {
//     ContactEventNumber(u.to_le_bytes())
// }

fn cens_in_interval(start: i64, end: i64) -> Res<HashMap<u128, i64>> {
    let mut out = HashMap::new();

    let items = DB
        .get()
        .ok_or(DB_UNINIT)?
        .range::<i64, u128, _>(CENS_BY_TS, start..end)?;

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
