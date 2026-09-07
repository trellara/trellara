mod module_manifest_core;
mod module_manifest_product;
mod module_manifest_runtime;

pub use exports::*;
pub(crate) use module_manifest_core::*;
pub(crate) use module_manifest_product::*;
pub(crate) use module_manifest_runtime::*;

#[cfg(test)]
mod tests;
