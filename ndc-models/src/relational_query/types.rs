use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
        scale: u8,
        prec: i8,
    },
    /// 256-bit decimal
    Decimal256 {
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

#[derive(Debug, Clone, PartialOrd, Serialize, Deserialize, JsonSchema)]
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
    /// 256-bit decimal
    Decimal256 {
        // These are strings to avoid more work for connector authors decoding separate high and low bits.
        value: String,
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

// Manually implemented so that float values are compared using their binary representation
#[allow(clippy::match_same_arms)]
impl PartialEq for RelationalLiteral {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Null, _) => false,
            (Self::Boolean { value: l_value }, Self::Boolean { value: r_value }) => {
                l_value == r_value
            }
            (Self::Boolean { .. }, _) => false,
            (Self::String { value: l_value }, Self::String { value: r_value }) => {
                l_value == r_value
            }
            (Self::String { .. }, _) => false,
            (Self::Int8 { value: l_value }, Self::Int8 { value: r_value }) => l_value == r_value,
            (Self::Int8 { .. }, _) => false,
            (Self::Int16 { value: l_value }, Self::Int16 { value: r_value }) => l_value == r_value,
            (Self::Int16 { .. }, _) => false,
            (Self::Int32 { value: l_value }, Self::Int32 { value: r_value }) => l_value == r_value,
            (Self::Int32 { .. }, _) => false,
            (Self::Int64 { value: l_value }, Self::Int64 { value: r_value }) => l_value == r_value,
            (Self::Int64 { .. }, _) => false,
            (Self::UInt8 { value: l_value }, Self::UInt8 { value: r_value }) => l_value == r_value,
            (Self::UInt8 { .. }, _) => false,
            (Self::UInt16 { value: l_value }, Self::UInt16 { value: r_value }) => {
                l_value == r_value
            }
            (Self::UInt16 { .. }, _) => false,
            (Self::UInt32 { value: l_value }, Self::UInt32 { value: r_value }) => {
                l_value == r_value
            }
            (Self::UInt32 { .. }, _) => false,
            (Self::UInt64 { value: l_value }, Self::UInt64 { value: r_value }) => {
                l_value == r_value
            }
            (Self::UInt64 { .. }, _) => false,
            (Self::Float32 { value: l_value }, Self::Float32 { value: r_value }) => {
                l_value.to_bits() == r_value.to_bits()
            }
            (Self::Float32 { .. }, _) => false,
            (Self::Float64 { value: l_value }, Self::Float64 { value: r_value }) => {
                l_value.to_bits() == r_value.to_bits()
            }
            (Self::Float64 { .. }, _) => false,
            (
                Self::Decimal128 {
                    value: l_value,
                    scale: l_scale,
                    prec: l_prec,
                },
                Self::Decimal128 {
                    value: r_value,
                    scale: r_scale,
                    prec: r_prec,
                },
            ) => l_value == r_value && l_scale == r_scale && l_prec == r_prec,
            (Self::Decimal128 { .. }, _) => false,
            (
                Self::Decimal256 {
                    value: l_value,
                    scale: l_scale,
                    prec: l_prec,
                },
                Self::Decimal256 {
                    value: r_value,
                    scale: r_scale,
                    prec: r_prec,
                },
            ) => l_value == r_value && l_scale == r_scale && l_prec == r_prec,
            (Self::Decimal256 { .. }, _) => false,
            (Self::Date32 { value: l_value }, Self::Date32 { value: r_value }) => {
                l_value == r_value
            }
            (Self::Date32 { .. }, _) => false,
            (Self::Date64 { value: l_value }, Self::Date64 { value: r_value }) => {
                l_value == r_value
            }
            (Self::Date64 { .. }, _) => false,
            (Self::Time32Second { value: l_value }, Self::Time32Second { value: r_value }) => {
                l_value == r_value
            }
            (Self::Time32Second { .. }, _) => false,
            (
                Self::Time32Millisecond { value: l_value },
                Self::Time32Millisecond { value: r_value },
            ) => l_value == r_value,
            (Self::Time32Millisecond { .. }, _) => false,
            (
                Self::Time64Microsecond { value: l_value },
                Self::Time64Microsecond { value: r_value },
            ) => l_value == r_value,
            (Self::Time64Microsecond { .. }, _) => false,
            (
                Self::Time64Nanosecond { value: l_value },
                Self::Time64Nanosecond { value: r_value },
            ) => l_value == r_value,
            (Self::Time64Nanosecond { .. }, _) => false,
            (
                Self::TimestampSecond { value: l_value },
                Self::TimestampSecond { value: r_value },
            ) => l_value == r_value,
            (Self::TimestampSecond { .. }, _) => false,
            (
                Self::TimestampMillisecond { value: l_value },
                Self::TimestampMillisecond { value: r_value },
            ) => l_value == r_value,
            (Self::TimestampMillisecond { .. }, _) => false,
            (
                Self::TimestampMicrosecond { value: l_value },
                Self::TimestampMicrosecond { value: r_value },
            ) => l_value == r_value,
            (Self::TimestampMicrosecond { .. }, _) => false,
            (
                Self::TimestampNanosecond { value: l_value },
                Self::TimestampNanosecond { value: r_value },
            ) => l_value == r_value,
            (Self::TimestampNanosecond { .. }, _) => false,
            (Self::DurationSecond { value: l_value }, Self::DurationSecond { value: r_value }) => {
                l_value == r_value
            }
            (Self::DurationSecond { .. }, _) => false,
            (
                Self::DurationMillisecond { value: l_value },
                Self::DurationMillisecond { value: r_value },
            ) => l_value == r_value,
            (Self::DurationMillisecond { .. }, _) => false,
            (
                Self::DurationMicrosecond { value: l_value },
                Self::DurationMicrosecond { value: r_value },
            ) => l_value == r_value,
            (Self::DurationMicrosecond { .. }, _) => false,
            (
                Self::DurationNanosecond { value: l_value },
                Self::DurationNanosecond { value: r_value },
            ) => l_value == r_value,
            (Self::DurationNanosecond { .. }, _) => false,
        }
    }
}

impl Eq for RelationalLiteral {}

// Manually implemented so that float values are hashed with their binary representation
impl std::hash::Hash for RelationalLiteral {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            RelationalLiteral::Null => 0.hash(state),
            RelationalLiteral::Boolean { value } => {
                1.hash(state);
                value.hash(state);
            }
            RelationalLiteral::Float32 { value } => {
                2.hash(state);
                value.to_bits().hash(state);
            }
            RelationalLiteral::Float64 { value } => {
                3.hash(state);
                value.to_bits().hash(state);
            }
            RelationalLiteral::Int8 { value } => {
                4.hash(state);
                value.hash(state);
            }
            RelationalLiteral::Int16 { value } => {
                5.hash(state);
                value.hash(state);
            }
            RelationalLiteral::Int32 { value } => {
                6.hash(state);
                value.hash(state);
            }
            RelationalLiteral::Int64 { value } => {
                7.hash(state);
                value.hash(state);
            }
            RelationalLiteral::UInt8 { value } => {
                8.hash(state);
                value.hash(state);
            }
            RelationalLiteral::UInt16 { value } => {
                9.hash(state);
                value.hash(state);
            }
            RelationalLiteral::UInt32 { value } => {
                10.hash(state);
                value.hash(state);
            }
            RelationalLiteral::UInt64 { value } => {
                11.hash(state);
                value.hash(state);
            }
            RelationalLiteral::Decimal128 { value, scale, prec } => {
                12.hash(state);
                value.hash(state);
                scale.hash(state);
                prec.hash(state);
            }
            RelationalLiteral::Decimal256 { value, scale, prec } => {
                13.hash(state);
                value.hash(state);
                scale.hash(state);
                prec.hash(state);
            }
            RelationalLiteral::String { value } => {
                14.hash(state);
                value.hash(state);
            }
            RelationalLiteral::Date32 { value } => {
                15.hash(state);
                value.hash(state);
            }
            RelationalLiteral::Date64 { value } => {
                16.hash(state);
                value.hash(state);
            }
            RelationalLiteral::Time32Second { value } => {
                17.hash(state);
                value.hash(state);
            }
            RelationalLiteral::Time32Millisecond { value } => {
                18.hash(state);
                value.hash(state);
            }
            RelationalLiteral::Time64Microsecond { value } => {
                19.hash(state);
                value.hash(state);
            }
            RelationalLiteral::Time64Nanosecond { value } => {
                20.hash(state);
                value.hash(state);
            }
            RelationalLiteral::TimestampSecond { value } => {
                21.hash(state);
                value.hash(state);
            }
            RelationalLiteral::TimestampMillisecond { value } => {
                22.hash(state);
                value.hash(state);
            }
            RelationalLiteral::TimestampMicrosecond { value } => {
                23.hash(state);
                value.hash(state);
            }
            RelationalLiteral::TimestampNanosecond { value } => {
                24.hash(state);
                value.hash(state);
            }
            RelationalLiteral::DurationSecond { value } => {
                25.hash(state);
                value.hash(state);
            }
            RelationalLiteral::DurationMillisecond { value } => {
                26.hash(state);
                value.hash(state);
            }
            RelationalLiteral::DurationMicrosecond { value } => {
                27.hash(state);
                value.hash(state);
            }
            RelationalLiteral::DurationNanosecond { value } => {
                28.hash(state);
                value.hash(state);
            }
        }
    }
}
