//! Audited FreeFlow model catalog and immutable install manifests.
//!
//! A catalog entry is downloadable only when it has a corresponding manifest
//! here. Runtime code never accepts a caller-supplied URL, size, digest, license,
//! or destination. That keeps the user confirmation screen and the bytes the
//! backend fetches bound to one reviewed record.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use std::path::Path;

use crate::managers::model::{EngineType, ModelDescriptor, ModelSource, QuantFile};
use crate::managers::model_capabilities::{CapabilityProbe, Compatibility};

pub const PARAKEET_MODEL_ID: &str = "parakeet-unified-en-0.6b-q8_0";

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
pub struct ModelLicenseDisclosure {
    pub scope: String,
    pub name: String,
    pub identifier: String,
    pub url: String,
    pub attribution: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelArtifactManifest {
    pub schema_version: u32,
    pub model_id: String,
    pub display_name: String,
    pub description: String,
    pub artifact_repository: String,
    pub artifact_revision: String,
    pub source_url: String,
    pub base_repository: String,
    pub base_revision: String,
    pub filename: String,
    pub format: String,
    pub quantization: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub licenses: Vec<ModelLicenseDisclosure>,
    pub redistribution_status: String,
}

impl ModelArtifactManifest {
    pub fn digest(&self) -> String {
        let encoded = serde_json::to_vec(self).expect("static model manifest must serialize");
        format!("{:x}", Sha256::digest(encoded))
    }

    pub fn receipt_filename(&self) -> String {
        format!("{}.freeflow-model.json", self.filename)
    }

    pub fn destination(&self, models_dir: &Path) -> std::path::PathBuf {
        models_dir.join(&self.filename)
    }
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct ModelInstallPlan {
    pub schema_version: u32,
    pub model_id: String,
    pub display_name: String,
    pub source_url: String,
    pub artifact_repository: String,
    pub artifact_revision: String,
    pub base_repository: String,
    pub base_revision: String,
    pub filename: String,
    pub format: String,
    pub quantization: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub destination: String,
    pub licenses: Vec<ModelLicenseDisclosure>,
    pub redistribution_status: String,
    pub manifest_digest: String,
}

impl ModelInstallPlan {
    pub fn from_manifest(manifest: &ModelArtifactManifest, models_dir: &Path) -> Self {
        Self {
            schema_version: manifest.schema_version,
            model_id: manifest.model_id.clone(),
            display_name: manifest.display_name.clone(),
            source_url: manifest.source_url.clone(),
            artifact_repository: manifest.artifact_repository.clone(),
            artifact_revision: manifest.artifact_revision.clone(),
            base_repository: manifest.base_repository.clone(),
            base_revision: manifest.base_revision.clone(),
            filename: manifest.filename.clone(),
            format: manifest.format.clone(),
            quantization: manifest.quantization.clone(),
            size_bytes: manifest.size_bytes,
            sha256: manifest.sha256.clone(),
            destination: manifest.destination(models_dir).display().to_string(),
            licenses: manifest.licenses.clone(),
            redistribution_status: manifest.redistribution_status.clone(),
            manifest_digest: manifest.digest(),
        }
    }
}

pub static MANIFESTS: Lazy<Vec<ModelArtifactManifest>> = Lazy::new(|| {
    let manifest: ModelArtifactManifest = serde_json::from_str(include_str!(
        "../../../models/manifests/parakeet-unified-en-0.6b-q8_0.json"
    ))
    .expect("checked-in Parakeet manifest must parse");
    assert_eq!(manifest.model_id, PARAKEET_MODEL_ID);
    validate_manifest(&manifest)
        .expect("checked-in Parakeet manifest must be immutable and complete");
    vec![manifest]
});

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub fn validate_manifest(manifest: &ModelArtifactManifest) -> Result<(), String> {
    if manifest.schema_version != 1 {
        return Err("unsupported model manifest schema".to_string());
    }
    if !is_lower_hex(&manifest.artifact_revision, 40)
        || !is_lower_hex(&manifest.base_revision, 40)
        || !is_lower_hex(&manifest.sha256, 64)
    {
        return Err("model revisions and SHA-256 must be full lowercase hex pins".to_string());
    }
    if manifest.size_bytes == 0 || manifest.licenses.is_empty() {
        return Err("model size and license disclosures are required".to_string());
    }
    let expected_url = format!(
        "https://huggingface.co/{}/resolve/{}/{}",
        manifest.artifact_repository, manifest.artifact_revision, manifest.filename
    );
    if manifest.source_url != expected_url {
        return Err("model source URL must be derived from its pinned Hugging Face repository, revision, and filename".to_string());
    }
    Ok(())
}

pub fn manifest_for(model_id: &str) -> Option<&'static ModelArtifactManifest> {
    MANIFESTS
        .iter()
        .find(|manifest| manifest.model_id == model_id)
}

pub fn accepted_manifest(
    model_id: &str,
    accepted_manifest_digest: &str,
) -> Result<&'static ModelArtifactManifest, String> {
    let manifest = manifest_for(model_id)
        .ok_or_else(|| format!("Model has no approved install manifest: {model_id}"))?;
    if accepted_manifest_digest != manifest.digest() {
        return Err("Model install confirmation is missing or stale; review the current source, size, hash, license, and destination before retrying".to_string());
    }
    Ok(manifest)
}

pub static CATALOG: Lazy<Vec<ModelDescriptor>> = Lazy::new(|| {
    MANIFESTS
        .iter()
        .map(|manifest| ModelDescriptor {
            id: manifest.model_id.clone(),
            source: ModelSource::Manifest {
                manifest_id: manifest.model_id.clone(),
            },
            name: manifest.display_name.clone(),
            description: manifest.description.clone(),
            engine_type: EngineType::TranscribeCpp,
            caps: CapabilityProbe {
                verdict: Compatibility::Compatible,
                display_name: Some(manifest.display_name.clone()),
                architecture: Some("parakeet".to_string()),
                variant: Some("unified-en-0.6b".to_string()),
                languages: Some(vec!["en".to_string()]),
                supports_streaming: Some(true),
                supports_translation: Some(false),
                supports_language_detect: Some(false),
            },
            files: vec![QuantFile {
                filename: manifest.filename.clone(),
                quant: manifest.quantization.clone(),
                size_bytes: manifest.size_bytes,
            }],
            default_quant: Some(manifest.quantization.clone()),
            speed_score: 0.79,
            accuracy_score: 0.90,
            recommended_rank: Some(1),
            recommended: true,
        })
        .collect()
});

pub fn rank_of(model_id: &str) -> u32 {
    CATALOG
        .iter()
        .find(|descriptor| descriptor.id == model_id)
        .and_then(|descriptor| descriptor.recommended_rank)
        .unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalog_model_has_a_complete_immutable_manifest() {
        assert!(!CATALOG.is_empty());
        for descriptor in CATALOG.iter() {
            let manifest = manifest_for(&descriptor.id).expect("catalog manifest");
            assert_eq!(manifest.artifact_revision.len(), 40);
            assert_eq!(manifest.base_revision.len(), 40);
            assert_eq!(manifest.sha256.len(), 64);
            assert!(manifest.size_bytes > 0);
            assert!(manifest.source_url.contains(&manifest.artifact_revision));
            assert!(manifest.source_url.ends_with(&manifest.filename));
            assert!(manifest.licenses.len() >= 2);
        }
    }

    #[test]
    fn install_plan_digest_changes_if_security_critical_data_changes() {
        let manifest = manifest_for(PARAKEET_MODEL_ID).expect("Parakeet manifest");
        let original = manifest.digest();
        let mut changed = manifest.clone();
        changed.size_bytes += 1;
        assert_ne!(original, changed.digest());
        changed = manifest.clone();
        changed.source_url.push_str("?changed");
        assert_ne!(original, changed.digest());
    }

    #[test]
    fn stale_or_missing_consent_digest_is_rejected() {
        let manifest = manifest_for(PARAKEET_MODEL_ID).expect("Parakeet manifest");
        assert!(accepted_manifest(PARAKEET_MODEL_ID, "").is_err());
        assert!(accepted_manifest(PARAKEET_MODEL_ID, &manifest.digest()).is_ok());
    }
}
