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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_timestamp_extracts_seconds_and_subsecond_nanos() {
        let dt = Utc
            .timestamp_opt(1_700_000_000, 123_456_789)
            .single()
            .unwrap();

        let ts = to_timestamp(dt);

        assert_eq!((ts.seconds, ts.nanos), (1_700_000_000, 123_456_789));
    }

    #[test]
    fn from_timestamp_reverses_to_timestamp_preserving_nanos() {
        let original = Utc
            .timestamp_opt(1_700_000_000, 987_654_321)
            .single()
            .unwrap();

        let round_tripped = from_timestamp(&to_timestamp(original)).unwrap();

        assert_eq!(round_tripped, original);
    }

    #[test]
    fn from_timestamp_rejects_negative_nanos() {
        let result = from_timestamp(&Timestamp {
            seconds: 0,
            nanos: -1,
        });

        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn from_timestamp_rejects_out_of_range_seconds() {
        let result = from_timestamp(&Timestamp {
            seconds: i64::MAX,
            nanos: 0,
        });

        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }
}
