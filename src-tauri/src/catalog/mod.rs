//! FreeFlow model catalog seam.
//!
//! FF-G3 deliberately ships an empty downloadable catalog. FF-V1 will populate
//! this seam only after every model source has an immutable revision, byte size,
//! SHA-256, license, attribution, and redistribution decision.

use once_cell::sync::Lazy;

use crate::managers::model::ModelDescriptor;

pub static CATALOG: Lazy<Vec<ModelDescriptor>> = Lazy::new(Vec::new);

pub fn rank_of(_model_id: &str) -> u32 {
    u32::MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foundation_catalog_is_intentionally_empty() {
        assert!(CATALOG.is_empty());
    }
}
