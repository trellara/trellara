use super::*;

mod canonicalization;
mod lookup_validation;
mod pending;
mod rejected;
mod released;
mod unexpected;
mod validation;

fn ddl_flow() -> DdlBarrierLookup {
    DdlBarrierLookup::new("source-a", "retail", "sales")
}
