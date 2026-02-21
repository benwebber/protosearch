mod context;
mod dialect;
mod error;
mod message;
mod plugin;

pub use error::{Error, Result};
pub use plugin::process;

pub const EXTENSION_NUMBER: u32 = 50_000;
pub const EXTENSION_MESSAGE_NAME: &str = "protosearch.FieldMappingOptions";
