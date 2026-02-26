pub mod transport;
pub mod kong_tools;
pub mod core;
pub mod error;
pub mod util;

pub use kong_tools::{KingKong, Kong, Monkey};
pub use error::NSLibError;
pub use util::boxed_future_generator;

// Type aliases
pub type Error = NSLibError;

// Re-export protocol types from simian-reactor for backward compatibility
pub use simian_reactor::protocol::{Frame, Proc};

// Re-export empty marker traits for backward compatibility
// Note: These traits don't provide methods - use Frame::encode() and Frame::decode() directly
pub trait Encoder {}
pub trait Decoder {}