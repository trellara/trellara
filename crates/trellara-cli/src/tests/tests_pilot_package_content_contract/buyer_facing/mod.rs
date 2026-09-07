use super::*;

mod ddl_proofs;
mod evidence_brief;
mod readme_inventory;

impl PilotPackageContents {
    pub(super) fn assert_buyer_facing_contents(&self) {
        readme_inventory::assert_readme_inventory(self);
        evidence_brief::assert_evidence_brief(self);
        ddl_proofs::assert_ddl_proofs(self);
    }
}
