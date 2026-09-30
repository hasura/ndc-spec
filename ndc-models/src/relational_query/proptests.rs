//! Property-based JSON round-trip tests for the relational query model types.

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use std::fmt::Debug;

use arbitrary::{Arbitrary, Unstructured};
use serde::{de::DeserializeOwned, Serialize};

use super::{Float32, Float64};

const SEEDS: usize = 256;
// Bounds the nesting depth of generated values: derive consumes ~4 bytes per enum
// level, so 256 bytes caps depth near 64, which keeps tests fast and under
// serde_json's default deserialization depth limit.
const BUFFER_SIZE: usize = 256;
// Backstop so a zero-consumption generator can't spin on one buffer forever.
const MAX_VALUES_PER_SEED: usize = 64;

// serde_json can't round-trip non-finite floats (written as `null`) or some
// extreme/subnormal values, so restrict to floats that survive; others become 0.0.
fn json_round_trippable_f32(value: f32) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    let survives = serde_json::to_string(&value)
        .ok()
        .and_then(|json| serde_json::from_str::<f32>(&json).ok())
        .is_some_and(|parsed| parsed.to_bits() == value.to_bits());
    if survives {
        value
    } else {
        0.0
    }
}

fn json_round_trippable_f64(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    let survives = serde_json::to_string(&value)
        .ok()
        .and_then(|json| serde_json::from_str::<f64>(&json).ok())
        .is_some_and(|parsed| parsed.to_bits() == value.to_bits());
    if survives {
        value
    } else {
        0.0
    }
}

impl<'a> Arbitrary<'a> for Float32 {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Float32(json_round_trippable_f32(f32::arbitrary(u)?)))
    }
}

impl<'a> Arbitrary<'a> for Float64 {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Float64(json_round_trippable_f64(f64::arbitrary(u)?)))
    }
}

// Hand-written (not derived) to exclude `Decimal128`, whose `i128` field can't be
// deserialized inside an internally-tagged enum (serde's `Content` rejects `i128`);
// see `decimal128_literal_is_a_known_non_round_trip`. New variants must be added here.
impl<'a> Arbitrary<'a> for crate::RelationalLiteral {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        use crate::RelationalLiteral as Lit;
        Ok(match u.int_in_range(0u32..=28)? {
            0 => Lit::Null,
            1 => Lit::Boolean {
                value: bool::arbitrary(u)?,
            },
            2 => Lit::String {
                value: String::arbitrary(u)?,
            },
            3 => Lit::Int8 {
                value: i8::arbitrary(u)?,
            },
            4 => Lit::Int16 {
                value: i16::arbitrary(u)?,
            },
            5 => Lit::Int32 {
                value: i32::arbitrary(u)?,
            },
            6 => Lit::Int64 {
                value: i64::arbitrary(u)?,
            },
            7 => Lit::UInt8 {
                value: u8::arbitrary(u)?,
            },
            8 => Lit::UInt16 {
                value: u16::arbitrary(u)?,
            },
            9 => Lit::UInt32 {
                value: u32::arbitrary(u)?,
            },
            10 => Lit::UInt64 {
                value: u64::arbitrary(u)?,
            },
            11 => Lit::Float32 {
                value: Float32::arbitrary(u)?,
            },
            12 => Lit::Float64 {
                value: Float64::arbitrary(u)?,
            },
            // Decimal128 intentionally omitted (see above).
            13 => Lit::Decimal256 {
                value: String::arbitrary(u)?,
                scale: i8::arbitrary(u)?,
                prec: u8::arbitrary(u)?,
            },
            14 => Lit::Date32 {
                value: i32::arbitrary(u)?,
            },
            15 => Lit::Date64 {
                value: i64::arbitrary(u)?,
            },
            16 => Lit::Time32Second {
                value: i32::arbitrary(u)?,
            },
            17 => Lit::Time32Millisecond {
                value: i32::arbitrary(u)?,
            },
            18 => Lit::Time64Microsecond {
                value: i64::arbitrary(u)?,
            },
            19 => Lit::Time64Nanosecond {
                value: i64::arbitrary(u)?,
            },
            20 => Lit::TimestampSecond {
                value: i64::arbitrary(u)?,
            },
            21 => Lit::TimestampMillisecond {
                value: i64::arbitrary(u)?,
            },
            22 => Lit::TimestampMicrosecond {
                value: i64::arbitrary(u)?,
            },
            23 => Lit::TimestampNanosecond {
                value: i64::arbitrary(u)?,
            },
            24 => Lit::DurationSecond {
                value: i64::arbitrary(u)?,
            },
            25 => Lit::DurationMillisecond {
                value: i64::arbitrary(u)?,
            },
            26 => Lit::DurationMicrosecond {
                value: i64::arbitrary(u)?,
            },
            27 => Lit::DurationNanosecond {
                value: i64::arbitrary(u)?,
            },
            _ => Lit::Interval {
                months: i32::arbitrary(u)?,
                days: i32::arbitrary(u)?,
                nanoseconds: i64::arbitrary(u)?,
            },
        })
    }
}

// A fixed pool rather than fuzzing the full string space; the awkward entries
// (empty, dotted, spaced, unicode) exercise the JSON string-escaping paths.
const SAMPLE_NAMES: &[&str] = &["", "a", "col_1", "Namespace.Table", "with space", "🦀"];

macro_rules! arbitrary_name_newtype {
    ($ty:ty) => {
        impl<'a> Arbitrary<'a> for $ty {
            fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
                Ok(<$ty>::from(*u.choose(SAMPLE_NAMES)?))
            }
        }
    };
}

arbitrary_name_newtype!(crate::CollectionName);
arbitrary_name_newtype!(crate::FieldName);
arbitrary_name_newtype!(crate::ArgumentName);

/// Asserts every generated `T` survives encode -> decode unchanged, and that
/// re-encoding the decoded value is byte-identical (a normalized wire format).
fn assert_round_trips<T>()
where
    T: for<'a> Arbitrary<'a> + Serialize + DeserializeOwned + PartialEq + Debug,
{
    let type_name = std::any::type_name::<T>();
    let mut generated = 0usize;

    for seed_index in 0..SEEDS {
        let buffer = high_entropy_seed(type_name, seed_index);
        let mut unstructured = Unstructured::new(&buffer);

        for _ in 0..MAX_VALUES_PER_SEED {
            let remaining_before = unstructured.len();
            // `Err` means the buffer is exhausted for this seed.
            let Ok(value) = T::arbitrary(&mut unstructured) else {
                break;
            };
            generated += 1;

            let encoded = serde_json::to_string(&value)
                .unwrap_or_else(|err| panic!("failed to serialize {type_name}: {err}"));
            let decoded: T = serde_json::from_str(&encoded).unwrap_or_else(|err| {
                panic!("failed to deserialize {type_name} from `{encoded}`: {err}")
            });
            assert_eq!(
                value, decoded,
                "encode -> decode changed a {type_name} value; json = {encoded}"
            );

            let reencoded = serde_json::to_string(&decoded)
                .unwrap_or_else(|err| panic!("failed to re-serialize {type_name}: {err}"));
            assert_eq!(
                encoded, reencoded,
                "decode -> encode was not idempotent for {type_name}"
            );

            // Stop if the generator consumed nothing (zero-sized type), to avoid spinning.
            if unstructured.len() == remaining_before {
                break;
            }
        }
    }

    assert!(
        generated > 0,
        "generated no arbitrary values for {type_name}; increase BUFFER_SIZE"
    );
}

/// Deterministic per-type seed bytes (an LCG), so runs are reproducible in CI.
fn high_entropy_seed(type_name: &str, seed_index: usize) -> Vec<u8> {
    let mut state = type_name
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |acc, byte| {
            (acc ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    state = state
        .wrapping_add((seed_index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
        .wrapping_add(1);

    let mut buffer = Vec::with_capacity(BUFFER_SIZE);
    for _ in 0..BUFFER_SIZE {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        buffer.push((state >> 33) as u8);
    }
    buffer
}

/// Runs on a large stack: deeply nested generated values can overflow the default
/// test-thread stack when (de)serialized or dropped.
fn with_large_stack(test: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(test)
        .expect("failed to spawn property-test thread")
        .join()
        .expect("property-test thread panicked");
}

// The type the review on #259 concerns; this property replaces the per-variant
// example tests and covers every variant reachable from it.
#[test]
fn relational_expression_round_trips() {
    with_large_stack(assert_round_trips::<super::RelationalExpression>);
}

#[test]
fn relational_literal_round_trips() {
    with_large_stack(assert_round_trips::<super::RelationalLiteral>);
}

#[test]
fn cast_type_round_trips() {
    with_large_stack(assert_round_trips::<super::CastType>);
}

#[test]
fn sort_round_trips() {
    with_large_stack(assert_round_trips::<super::Sort>);
}

#[test]
fn case_when_round_trips() {
    with_large_stack(assert_round_trips::<super::CaseWhen>);
}

#[test]
fn date_part_unit_round_trips() {
    with_large_stack(assert_round_trips::<super::DatePartUnit>);
}

#[test]
fn relation_round_trips() {
    with_large_stack(assert_round_trips::<super::Relation>);
}

#[test]
fn join_on_round_trips() {
    with_large_stack(assert_round_trips::<super::JoinOn>);
}

#[test]
fn relational_query_capabilities_round_trips() {
    with_large_stack(assert_round_trips::<crate::RelationalQueryCapabilities>);
}

// Pins the known limitation (see the `RelationalLiteral` generator above): if this
// ever round-trips, drop the `Decimal128` exclusion there and delete this test.
#[test]
fn decimal128_literal_is_a_known_non_round_trip() {
    let literal = crate::RelationalLiteral::Decimal128 {
        value: 1,
        scale: 0,
        prec: 1,
    };
    let json = serde_json::to_string(&literal).expect("Decimal128 should still serialize");
    let decoded = serde_json::from_str::<crate::RelationalLiteral>(&json);
    assert!(
        decoded.is_err(),
        "Decimal128 unexpectedly round-tripped ({json}); it is now safe to include \
         it in the RelationalLiteral fuzz generator in proptests.rs and delete this pin"
    );
}
