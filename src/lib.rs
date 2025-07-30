pub mod config;
pub mod error;
pub mod mcp;
pub mod tools;

pub use error::AppError;
pub type AppResult<T> = std::result::Result<T, AppError>;
