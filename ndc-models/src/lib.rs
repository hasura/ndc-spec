mod aggregation;
pub use aggregation::*;
mod capabilities;
pub use capabilities::*;
mod expression;
pub use expression::*;
mod fields;
pub use fields::*;
mod names;
pub use names::*;
mod ordering;
pub use ordering::*;
mod requests;
pub use requests::*;
mod schema;
pub use schema::*;

pub const VERSION_HEADER_NAME: &str = "X-Hasura-NDC-Version";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
