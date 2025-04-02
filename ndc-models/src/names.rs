use ref_cast::RefCast;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::borrow::Borrow;

macro_rules! newtype {
    ($name: ident over $oldtype: ident) => {
        #[derive(
            Clone,
            Debug,
            Default,
            Hash,
            Eq,
            Ord,
            PartialEq,
            PartialOrd,
            Serialize,
            Deserialize,
            RefCast,
        )]
        #[repr(transparent)]
        pub struct $name($oldtype);

        impl JsonSchema for $name {
            fn schema_name() -> String {
                String::schema_name()
            }

            fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
                String::json_schema(gen)
            }

            fn is_referenceable() -> bool {
                String::is_referenceable()
            }

            fn schema_id() -> std::borrow::Cow<'static, str> {
                String::schema_id()
            }
        }

        impl AsRef<$oldtype> for $name {
            fn as_ref(&self) -> &$oldtype {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                $name(value.into())
            }
        }

        impl From<$oldtype> for $name {
            fn from(value: $oldtype) -> Self {
                $name(value)
            }
        }

        impl From<$name> for $oldtype {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl Borrow<str> for $name {
            fn borrow(&self) -> &str {
                self.0.as_str()
            }
        }

        impl Borrow<$oldtype> for $name {
            fn borrow(&self) -> &$oldtype {
                &self.0
            }
        }

        impl $name {
            pub fn new(value: $oldtype) -> Self {
                $name(value)
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            pub fn into_inner(self) -> $oldtype {
                self.0
            }

            pub fn inner(&self) -> &$oldtype {
                &self.0
            }
        }
    };
    ($name: ident) => {
        newtype! {$name over SmolStr}

        impl From<String> for $name {
            fn from(value: String) -> Self {
                $name(value.into())
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0.into()
            }
        }
    };
}

pub(crate) use newtype;

newtype! {AggregateFunctionName}
newtype! {ArgumentName}
newtype! {CollectionName}
newtype! {ComparisonOperatorName}
newtype! {ExtractionFunctionName}
newtype! {FieldName}
newtype! {FunctionName over CollectionName}
newtype! {ObjectTypeName over TypeName}
newtype! {ProcedureName}
newtype! {RelationshipName}
newtype! {ScalarTypeName over TypeName}
newtype! {TypeName}
newtype! {VariableName}

impl From<String> for FunctionName {
    fn from(value: String) -> Self {
        FunctionName(value.into())
    }
}

impl From<FunctionName> for String {
    fn from(value: FunctionName) -> Self {
        value.0.into()
    }
}

impl From<String> for ObjectTypeName {
    fn from(value: String) -> Self {
        ObjectTypeName(value.into())
    }
}

impl From<ObjectTypeName> for String {
    fn from(value: ObjectTypeName) -> Self {
        value.0.into()
    }
}

impl From<String> for ScalarTypeName {
    fn from(value: String) -> Self {
        ScalarTypeName(value.into())
    }
}

impl From<ScalarTypeName> for String {
    fn from(value: ScalarTypeName) -> Self {
        value.0.into()
    }
}
