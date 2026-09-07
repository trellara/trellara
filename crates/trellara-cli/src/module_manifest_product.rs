#[path = "module_manifest_product_fleet.rs"]
pub(crate) mod module_manifest_product_fleet;
#[path = "module_manifest_product_lake.rs"]
pub(crate) mod module_manifest_product_lake;
#[path = "module_manifest_product_pilot.rs"]
pub(crate) mod module_manifest_product_pilot;
#[path = "module_manifest_product_quickstart.rs"]
pub(crate) mod module_manifest_product_quickstart;
#[path = "module_manifest_product_workflows.rs"]
pub(crate) mod module_manifest_product_workflows;

pub(crate) use module_manifest_product_fleet::*;
pub(crate) use module_manifest_product_lake::*;
pub(crate) use module_manifest_product_pilot::*;
pub(crate) use module_manifest_product_quickstart::*;
pub(crate) use module_manifest_product_workflows::*;
