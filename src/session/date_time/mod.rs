use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};
use std::fmt;

pub fn serialize_human_readable<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = date.format("%Y-%m-%d %H:%M:%S").to_string();
    serializer.serialize_str(&s)
}

struct DateTimeVisitor;

impl<'de> Visitor<'de> for DateTimeVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representing a date and time in the format %Y-%m-%d %H:%M:%S")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S")
            .map(|naive| Utc.from_utc_datetime(&naive))
            .map_err(de::Error::custom)
    }
}

pub fn deserialize_human_readable<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(DateTimeVisitor)
}
