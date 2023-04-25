use crate::categorization::Categorization;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::time::Duration;

mod serde_duration {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let seconds = duration.as_secs();
        serializer.serialize_u64(seconds)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(seconds))
    }
}

//mod ts_seconds_option {
//    use chrono::{DateTime, NaiveDateTime, Utc};
//    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
//
//    pub fn serialize<S>(value: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
//    where
//        S: Serializer,
//    {
//        match value {
//            Some(dt) => dt.timestamp().serialize(serializer),
//            None => serializer.serialize_none(),
//        }
//    }
//
//    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
//    where
//        D: Deserializer<'de>,
//    {
//        let timestamp = Option::deserialize(deserializer)?;
//        timestamp.map_or(Ok(None), |ts| {
//            DateTime::from_utc(
//                NaiveDateTime::from_timestamp_opt(ts, 0)
//                    .ok_or_else(|| serde::de::Error::custom("invalid timestamp"))?,
//                Utc,
//            )
//            .map(serde::de::Error::custom)
//        })
//    }
//}

// TODO: Impl .duration()
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeChunk {
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub name: String,
    pub time_chunks: Vec<TimeChunk>,
    pub paused_duration: Duration,
    pub status: TaskStatus,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum TaskStatus {
    Running,
    Paused,
    Stopped,
}

// Update the TimePeriod struct
#[derive(Debug, Serialize, Deserialize)]
pub struct TimePeriod {
    pub categorization: Categorization,
}

impl Task {
    pub fn new(name: &str) -> Task {
        Task {
            name: name.to_string(),
            time_chunks: vec![TimeChunk {
                start_time: Utc::now(),
                end_time: None,
            }],
            paused_duration: Duration::from_secs(0),
            status: TaskStatus::Running,
        }
    }

    pub fn stop(&mut self) {
        if let Some(last_chunk) = self.time_chunks.last_mut() {
            last_chunk.end_time = Some(Utc::now());
        }
        self.status = TaskStatus::Stopped;
    }

    pub fn pause(&mut self) {
        if let TaskStatus::Running = self.status {
            if let Some(last_chunk) = self.time_chunks.last_mut() {
                last_chunk.end_time = Some(Utc::now());
            }
            self.status = TaskStatus::Paused;
        }
    }

    pub fn resume(&mut self) {
        if let TaskStatus::Paused = self.status {
            self.time_chunks.push(TimeChunk {
                start_time: Utc::now(),
                end_time: None,
            });
            self.status = TaskStatus::Running;
        }
    }

    pub fn time_spent(&self) {
        let total_duration = self
            .time_chunks
            .iter()
            .filter_map(|chunk| match (chunk.start_time, chunk.end_time) {
                (start, Some(end)) => Some(end - start),
                (start, None) if self.status == TaskStatus::Running => Some(Utc::now() - start),
                _ => None,
            })
            .fold(Duration::from_secs(0), |acc, duration| {
                let std_duration = Duration::from_secs(duration.num_seconds() as u64);
                acc + std_duration
            });

        let seconds = total_duration.as_secs();
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let remaining_seconds = seconds % 60;
        println!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, remaining_seconds,);
    }
}

pub fn load_data(path: &PathBuf) -> TimePeriod {
    let file = File::open(path);
    match file {
        Ok(file) => {
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_else(|_| TimePeriod {
                categorization: Categorization::new(),
            })
        }
        Err(_) => TimePeriod {
            categorization: Categorization::new(),
        },
    }
}

pub fn save_data(path: &PathBuf, time_period: &TimePeriod) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, time_period)?;
    Ok(())
}
