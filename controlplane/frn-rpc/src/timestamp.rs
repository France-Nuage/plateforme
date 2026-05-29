use chrono::{DateTime, TimeZone, Utc};
use prost_types::Timestamp;
use tonic::Status;

pub fn to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

pub fn from_timestamp(ts: &Timestamp) -> Result<DateTime<Utc>, Status> {
    let nanos = u32::try_from(ts.nanos)
        .map_err(|_| Status::invalid_argument(format!("negative nanos: {}", ts.nanos)))?;

    Utc.timestamp_opt(ts.seconds, nanos)
        .single()
        .ok_or_else(|| {
            Status::invalid_argument(format!(
                "invalid timestamp: seconds={}, nanos={}",
                ts.seconds, ts.nanos
            ))
        })
}
