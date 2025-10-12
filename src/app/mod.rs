mod clipboard;
mod command;
mod core;
mod edit;
mod explorer;
mod explorer_ops;
mod file;
mod navigation;
mod search;
mod substitute;

pub use core::*;

// Re-export config for use within app module
use crate::config;
pub(crate) use config::{ColorScheme, RcConfig};
