use super::tcn_batches_manager::TcnBatchesManager;
use crate::{
    byte_vec_to_16_byte_array, database::tcn_dao::TcnDao, errors::ServicesError, expect_log,
    reports_interval,
};
use log::*;
use reports_interval::UnixTime;
use std::sync::{Arc, Mutex};
use tcn::TemporaryContactNumber;
use timer::{Guard, Timer};

#[derive(Debug, PartialEq, Clone)]
pub struct ObservedTcn {
    pub tcn: TemporaryContactNumber,
    pub contact_start: UnixTime,
    pub contact_end: UnixTime,
    pub min_distance: f32,
    pub avg_distance: f32,
    pub total_count: usize, // Needed to calculate correctly average of averages (= average of single values)
}

pub trait ObservedTcnProcessor {
    fn save(&self, tcn_str: &str, distance: f32) -> Result<(), ServicesError>;
}

pub struct ObservedTcnProcessorImpl<T>
where
    T: 'static + TcnDao,
{
    tcn_batches_manager: Arc<TcnBatchesManager<T>>,
    _timer_data: TimerData,
}

struct TimerData {
    _timer: Arc<Mutex<Timer>>,
    _guard: Guard,
}

impl<T> ObservedTcnProcessorImpl<T>
where
    T: 'static + TcnDao,
{
    pub fn new(tcn_batches_manager: TcnBatchesManager<T>) -> ObservedTcnProcessorImpl<T> {
        let tcn_batches_manager = Arc::new(tcn_batches_manager);
        let instance = ObservedTcnProcessorImpl {
            tcn_batches_manager: tcn_batches_manager.clone(),
            _timer_data: Self::schedule_process_batches(tcn_batches_manager),
        };
        instance
    }

    fn schedule_process_batches(tcn_batches_manager: Arc<TcnBatchesManager<T>>) -> TimerData {
        let timer = Arc::new(Mutex::new(Timer::new()));
        TimerData {
            _timer: timer.clone(),
            _guard: timer.clone().lock().unwrap().schedule_repeating(
                chrono::Duration::seconds(10),
                move || {
                    let flush_res = tcn_batches_manager.flush();
                    expect_log!(flush_res, "Couldn't flush TCNs");
                },
            ),
        }
    }
}

impl<T> ObservedTcnProcessor for ObservedTcnProcessorImpl<T>
where
    T: TcnDao + Sync + Send,
{
    fn save(&self, tcn_str: &str, distance: f32) -> Result<(), ServicesError> {
        debug!("Recording a TCN {:?}, distance: {}", tcn_str, distance);

        let bytes_vec: Vec<u8> = hex::decode(tcn_str)?;
        let observed_tcn = ObservedTcn {
            tcn: TemporaryContactNumber(byte_vec_to_16_byte_array(bytes_vec)),
            contact_start: UnixTime::now(),
            contact_end: UnixTime::now(),
            min_distance: distance,
            avg_distance: distance,
            total_count: 1,
        };

        self.tcn_batches_manager.push(observed_tcn);

        Ok(())
    }
}
