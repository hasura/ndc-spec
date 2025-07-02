use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Add newtypes for f32 and f64, to derive Hash and PartialEq
#[derive(Debug, Copy, Clone, PartialOrd, Serialize, Deserialize)]
pub struct Float32(pub f32);

#[derive(Debug, Copy, Clone, PartialOrd, Serialize, Deserialize)]
pub struct Float64(pub f64);

impl PartialEq for Float32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for Float32 {}

impl std::hash::Hash for Float32 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl PartialEq for Float64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for Float64 {}

impl std::hash::Hash for Float64 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl JsonSchema for Float32 {
    fn is_referenceable() -> bool {
        false
    }

    fn schema_name() -> String {
        f32::schema_name()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        f32::json_schema(gen)
    }
}

impl JsonSchema for Float64 {
    fn is_referenceable() -> bool {
        false
    }

    fn schema_name() -> String {
        f64::schema_name()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        f64::json_schema(gen)
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type")]
#[schemars(title = "CastType", rename_all = "snake_case")]
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
    #[serde(rename = "uint8")]
    UInt8,
    /// unsigned 16bit int
    #[serde(rename = "uint16")]
    UInt16,
    /// unsigned 32bit int
    #[serde(rename = "uint32")]
    UInt32,
    /// unsigned 64bit int
    #[serde(rename = "uint64")]
    UInt64,
    /// 32bit float
    Float32,
    /// 64bit float
    Float64,
    /// 128-bit decimal
    Decimal128 {
        scale: i8,
        prec: u8,
    },
    /// 256-bit decimal
    Decimal256 {
        scale: i8,
        prec: u8,
    },
    /// date
    Date,
    /// time
    Time,
    /// ISO 8601 timestamp
    Timestamp,
    /// duration
    Duration,
}

#[derive(Debug, Clone, PartialOrd, Hash, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[schemars(title = "RelationalLiteral", rename_all = "snake_case")]
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
    #[serde(rename = "uint8")]
    UInt8 {
        value: u8,
    },
    /// unsigned 16bit int
    #[serde(rename = "uint16")]
    UInt16 {
        value: u16,
    },
    /// unsigned 32bit int
    #[serde(rename = "uint32")]
    UInt32 {
        value: u32,
    },
    /// unsigned 64bit int
    #[serde(rename = "uint64")]
    UInt64 {
        value: u64,
    },
    /// 32bit float
    Float32 {
        value: Float32,
    },
    /// 64bit float
    Float64 {
        value: Float64,
    },
    /// 128-bit decimal
    Decimal128 {
        value: i128,
        scale: i8,
        prec: u8,
    },
    /// 256-bit decimal
    Decimal256 {
        // These are strings to avoid more work for connector authors decoding separate high and low bits.
        value: String,
        scale: i8,
        prec: u8,
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
