use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[schemars(title = "CastType")]
pub enum CastType {
    Boolean,
    /// utf-8 encoded string.
    Utf8,
    /// signed 8bit int
    Int8,
    /// signed 16bit int
    Int16,
    /// signed 32bit int
    Int32,
    /// signed 64bit int
    Int64,
    /// unsigned 8bit int
    UInt8,
    /// unsigned 16bit int
    UInt16,
    /// unsigned 32bit int
    UInt32,
    /// unsigned 64bit int
    UInt64,
    /// 32bit float
    Float32,
    /// 64bit float
    Float64,
    /// 128-bit decimal
    Decimal128 {
        scale: u8,
        prec: i8,
    },
    /// Date stored as a signed 32bit int days since UNIX epoch 1970-01-01
    Date32,
    /// Date stored as a signed 64bit int milliseconds since UNIX epoch 1970-01-01
    Date64,
    /// Time stored as a signed 32bit int as seconds since midnight
    Time32Second,
    /// Time stored as a signed 32bit int as milliseconds since midnight
    Time32Millisecond,
    /// Time stored as a signed 64bit int as microseconds since midnight
    Time64Microsecond,
    /// Time stored as a signed 64bit int as nanoseconds since midnight
    Time64Nanosecond,
    /// Timestamp Second
    TimestampSecond,
    /// Timestamp Milliseconds UInt32
    TimestampMillisecond,
    /// Timestamp Microseconds UInt64
    TimestampMicrosecond,
    /// Timestamp Nanoseconds
    TimestampNanosecond,
    /// Duration in seconds
    DurationSecond,
    /// Duration in milliseconds
    DurationMillisecond,
    /// Duration in microseconds
    DurationMicrosecond,
    /// Duration in nanoseconds
    DurationNanosecond,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[schemars(title = "RelationalLiteral")]
pub enum RelationalLiteral {
    Null,
    Boolean {
        value: bool,
    },
    /// utf-8 encoded string.
    String {
        value: String,
    },
    /// signed 8bit int
    Int8 {
        value: i8,
    },
    /// signed 16bit int
    Int16 {
        value: i16,
    },
    /// signed 32bit int
    Int32 {
        value: i32,
    },
    /// signed 64bit int
    Int64 {
        value: i64,
    },
    /// unsigned 8bit int
    UInt8 {
        value: u8,
    },
    /// unsigned 16bit int
    UInt16 {
        value: u16,
    },
    /// unsigned 32bit int
    UInt32 {
        value: u32,
    },
    /// unsigned 64bit int
    UInt64 {
        value: u64,
    },
    /// 32bit float
    Float32 {
        value: f32,
    },
    /// 64bit float
    Float64 {
        value: f64,
    },
    /// 128-bit decimal
    Decimal128 {
        value: i128,
        scale: u8,
        prec: i8,
    },
    /// Date stored as a signed 32bit int days since UNIX epoch 1970-01-01
    Date32 {
        value: i32,
    },
    /// Date stored as a signed 64bit int milliseconds since UNIX epoch 1970-01-01
    Date64 {
        value: i64,
    },
    /// Time stored as a signed 32bit int as seconds since midnight
    Time32Second {
        value: i32,
    },
    /// Time stored as a signed 32bit int as milliseconds since midnight
    Time32Millisecond {
        value: i32,
    },
    /// Time stored as a signed 64bit int as microseconds since midnight
    Time64Microsecond {
        value: i64,
    },
    /// Time stored as a signed 64bit int as nanoseconds since midnight
    Time64Nanosecond {
        value: i64,
    },
    /// Timestamp Second
    TimestampSecond {
        value: i64,
    },
    /// Timestamp Milliseconds
    TimestampMillisecond {
        value: i64,
    },
    /// Timestamp Microseconds
    TimestampMicrosecond {
        value: i64,
    },
    /// Timestamp Nanoseconds
    TimestampNanosecond {
        value: i64,
    },
    /// Duration in seconds
    DurationSecond {
        value: i64,
    },
    /// Duration in milliseconds
    DurationMillisecond {
        value: i64,
    },
    /// Duration in microseconds
    DurationMicrosecond {
        value: i64,
    },
    /// Duration in nanoseconds
    DurationNanosecond {
        value: i64,
    },
}
