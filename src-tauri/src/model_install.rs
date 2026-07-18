use crate::catalog::ModelArtifactManifest;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use hf_hub::api::tokio::CancellationToken;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferOutcome {
    Complete,
    Cancelled,
    RestartRequired,
}

fn approved_delivery_host(host: &str) -> bool {
    host == "huggingface.co"
        || host.ends_with(".huggingface.co")
        || host == "hf.co"
        || host.ends_with(".hf.co")
}

pub fn approved_model_http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 {
                return attempt.error("too many approved model redirects");
            }
            let url = attempt.url();
            match (url.scheme(), url.host_str()) {
                ("https", Some(host)) if approved_delivery_host(host) => attempt.follow(),
                _ => attempt.error("model source redirected outside approved HTTPS delivery hosts"),
            }
        }))
        .build()
        .context("build restricted model HTTP client")
}

pub async fn transfer_to_partial<F>(
    client: &reqwest::Client,
    source_url: &str,
    partial: &Path,
    expected_size: u64,
    cancel: &CancellationToken,
    mut on_progress: F,
) -> Result<TransferOutcome>
where
    F: FnMut(u64, u64),
{
    let resume_from = partial
        .metadata()
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    if resume_from > expected_size {
        let _ = fs::remove_file(partial);
        return Err(anyhow::anyhow!(
            "Partial model file exceeds the approved {expected_size} byte artifact and was removed"
        ));
    }

    let mut request = client.get(source_url);
    if resume_from > 0 {
        request = request.header(reqwest::header::RANGE, format!("bytes={resume_from}-"));
    }
    let response = request
        .send()
        .await
        .context("request approved model artifact")?;

    if resume_from > 0 && response.status() == reqwest::StatusCode::OK {
        drop(response);
        fs::remove_file(partial)
            .with_context(|| format!("remove non-resumable partial {}", partial.display()))?;
        return Ok(TransferOutcome::RestartRequired);
    }
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download model: HTTP {}",
            response.status()
        ));
    }
    if resume_from > 0 {
        if response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(anyhow::anyhow!(
                "Resume request returned HTTP {} instead of 206",
                response.status()
            ));
        }
        let expected_prefix = format!("bytes {resume_from}-");
        let expected_suffix = format!("/{expected_size}");
        let content_range = response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| anyhow::anyhow!("Resume response omitted Content-Range"))?;
        if !content_range.starts_with(&expected_prefix)
            || !content_range.ends_with(&expected_suffix)
        {
            return Err(anyhow::anyhow!(
                "Resume response Content-Range mismatch: expected byte {resume_from} of {expected_size}, got {content_range}"
            ));
        }
    }

    let expected_response_bytes = expected_size - resume_from;
    if let Some(content_length) = response.content_length() {
        if content_length != expected_response_bytes {
            return Err(anyhow::anyhow!(
                "Approved model response length mismatch: expected {expected_response_bytes} bytes, server announced {content_length} bytes"
            ));
        }
    }

    let mut downloaded = resume_from;
    let mut stream = response.bytes_stream();
    let mut file = if resume_from > 0 {
        fs::OpenOptions::new().append(true).open(partial)?
    } else {
        File::create(partial)?
    };
    on_progress(downloaded, expected_size);

    loop {
        let next_chunk = tokio::select! {
            _ = cancel.cancelled() => return Ok(TransferOutcome::Cancelled),
            chunk = stream.next() => chunk,
        };
        let Some(chunk) = next_chunk else {
            break;
        };
        let chunk = chunk.context("read approved model response")?;
        file.write_all(&chunk)
            .with_context(|| format!("write model partial {}", partial.display()))?;
        downloaded = downloaded.saturating_add(chunk.len() as u64);
        if downloaded > expected_size {
            drop(file);
            let _ = fs::remove_file(partial);
            return Err(anyhow::anyhow!(
                "Approved model response exceeded {expected_size} bytes"
            ));
        }
        on_progress(downloaded, expected_size);
    }
    if cancel.is_cancelled() {
        return Ok(TransferOutcome::Cancelled);
    }
    file.flush()?;
    file.sync_all()?;
    if downloaded != expected_size {
        return Err(anyhow::anyhow!(
            "Download incomplete: expected {expected_size} bytes, got {downloaded} bytes; retry to resume"
        ));
    }
    Ok(TransferOutcome::Complete)
}

pub const FREE_SPACE_RESERVE_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct InstallReceipt {
    schema_version: u32,
    model_id: String,
    manifest_digest: String,
    filename: String,
    size_bytes: u64,
    sha256: String,
    install_method: String,
}

pub fn receipt_path(models_dir: &Path, manifest: &ModelArtifactManifest) -> PathBuf {
    models_dir.join(manifest.receipt_filename())
}

pub fn partial_path(models_dir: &Path, manifest: &ModelArtifactManifest) -> PathBuf {
    models_dir.join(format!("{}.partial", manifest.filename))
}

pub fn remaining_bytes(manifest: &ModelArtifactManifest, partial_size: u64) -> Result<u64> {
    manifest
        .size_bytes
        .checked_sub(partial_size)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Partial model file is larger than the approved artifact: {} > {} bytes",
                partial_size,
                manifest.size_bytes
            )
        })
}

pub fn ensure_available_space(available: u64, remaining: u64) -> Result<()> {
    let required = remaining.saturating_add(FREE_SPACE_RESERVE_BYTES);
    if available < required {
        return Err(anyhow::anyhow!(
            "Not enough free disk space for model installation: {} bytes available, {} bytes required (including safety reserve)",
            available,
            required
        ));
    }
    Ok(())
}

pub fn check_disk_space(
    models_dir: &Path,
    manifest: &ModelArtifactManifest,
    partial_size: u64,
) -> Result<()> {
    fs::create_dir_all(models_dir)
        .with_context(|| format!("create model directory {}", models_dir.display()))?;
    let remaining = remaining_bytes(manifest, partial_size)?;
    let available = fs2::available_space(models_dir)
        .with_context(|| format!("query free space for {}", models_dir.display()))?;
    ensure_available_space(available, remaining)
}

pub fn compute_sha256(path: &Path) -> Result<String> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .with_context(|| format!("read {}", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn verify_artifact(path: &Path, manifest: &ModelArtifactManifest) -> Result<()> {
    let actual_size = path
        .metadata()
        .with_context(|| format!("inspect {}", path.display()))?
        .len();
    if actual_size != manifest.size_bytes {
        return Err(anyhow::anyhow!(
            "Model size mismatch: expected {} bytes, got {} bytes",
            manifest.size_bytes,
            actual_size
        ));
    }
    let actual_sha256 = compute_sha256(path)?;
    if actual_sha256 != manifest.sha256 {
        return Err(anyhow::anyhow!(
            "Model SHA-256 mismatch: expected {}, got {}",
            manifest.sha256,
            actual_sha256
        ));
    }
    Ok(())
}

fn receipt_for(manifest: &ModelArtifactManifest, install_method: &str) -> InstallReceipt {
    InstallReceipt {
        schema_version: 1,
        model_id: manifest.model_id.clone(),
        manifest_digest: manifest.digest(),
        filename: manifest.filename.clone(),
        size_bytes: manifest.size_bytes,
        sha256: manifest.sha256.clone(),
        install_method: install_method.to_string(),
    }
}

pub fn write_receipt(
    models_dir: &Path,
    manifest: &ModelArtifactManifest,
    install_method: &str,
) -> Result<()> {
    fs::create_dir_all(models_dir)
        .with_context(|| format!("create model directory {}", models_dir.display()))?;
    let destination = receipt_path(models_dir, manifest);
    let temp = tempfile::NamedTempFile::new_in(models_dir)
        .with_context(|| format!("create receipt in {}", models_dir.display()))?;
    {
        let mut writer = BufWriter::new(temp.as_file());
        serde_json::to_writer_pretty(&mut writer, &receipt_for(manifest, install_method))
            .context("serialize model install receipt")?;
        writer.write_all(b"\n").context("finish model receipt")?;
        writer.flush().context("flush model receipt")?;
    }
    temp.as_file().sync_all().context("sync model receipt")?;
    if destination.exists() {
        fs::remove_file(&destination)
            .with_context(|| format!("replace model receipt {}", destination.display()))?;
    }
    temp.persist(&destination)
        .map_err(|error| error.error)
        .with_context(|| format!("persist model receipt {}", destination.display()))?;
    Ok(())
}

pub fn has_verified_receipt(models_dir: &Path, manifest: &ModelArtifactManifest) -> bool {
    let model_path = manifest.destination(models_dir);
    let Ok(metadata) = model_path.metadata() else {
        return false;
    };
    if !metadata.is_file() || metadata.len() != manifest.size_bytes {
        return false;
    }
    let Ok(bytes) = fs::read(receipt_path(models_dir, manifest)) else {
        return false;
    };
    let Ok(receipt) = serde_json::from_slice::<InstallReceipt>(&bytes) else {
        return false;
    };
    let receipt_matches = receipt == receipt_for(manifest, &receipt.install_method)
        && receipt.schema_version == 1
        && matches!(receipt.install_method.as_str(), "download" | "manual");
    receipt_matches
        && compute_sha256(&model_path)
            .map(|sha256| sha256 == manifest.sha256)
            .unwrap_or(false)
}

pub fn remove_receipt(models_dir: &Path, manifest: &ModelArtifactManifest) -> Result<()> {
    let path = receipt_path(models_dir, manifest);
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("remove model receipt {}", path.display()))?;
    }
    Ok(())
}

pub fn finalize_verified_partial(
    models_dir: &Path,
    manifest: &ModelArtifactManifest,
    install_method: &str,
) -> Result<PathBuf> {
    let partial = partial_path(models_dir, manifest);
    if let Err(error) = verify_artifact(&partial, manifest) {
        let _ = fs::remove_file(&partial);
        return Err(error);
    }
    commit_verified_partial(models_dir, manifest, install_method)
}

pub fn commit_verified_partial(
    models_dir: &Path,
    manifest: &ModelArtifactManifest,
    install_method: &str,
) -> Result<PathBuf> {
    let partial = partial_path(models_dir, manifest);
    let destination = manifest.destination(models_dir);
    if destination.exists() {
        fs::remove_file(&destination)
            .with_context(|| format!("replace model {}", destination.display()))?;
    }
    fs::rename(&partial, &destination).with_context(|| {
        format!(
            "atomically install model {} as {}",
            partial.display(),
            destination.display()
        )
    })?;
    if let Err(error) = write_receipt(models_dir, manifest, install_method) {
        let _ = fs::remove_file(&destination);
        return Err(error);
    }
    Ok(destination)
}

pub fn install_from_local_file(
    source: &Path,
    models_dir: &Path,
    manifest: &ModelArtifactManifest,
) -> Result<PathBuf> {
    let source = source
        .canonicalize()
        .with_context(|| format!("resolve local model {}", source.display()))?;
    if !source.is_file() {
        return Err(anyhow::anyhow!(
            "Manual model source is not a file: {}",
            source.display()
        ));
    }
    fs::create_dir_all(models_dir)
        .with_context(|| format!("create model directory {}", models_dir.display()))?;
    let partial = partial_path(models_dir, manifest);
    if source == partial || source == manifest.destination(models_dir) {
        return Err(anyhow::anyhow!(
            "Manual model source must be outside FreeFlow's managed destination"
        ));
    }
    check_disk_space(models_dir, manifest, 0)?;
    if partial.exists() {
        fs::remove_file(&partial)
            .with_context(|| format!("remove stale partial {}", partial.display()))?;
    }
    fs::copy(&source, &partial).with_context(|| {
        format!(
            "copy local model {} to {}",
            source.display(),
            partial.display()
        )
    })?;
    if let Err(error) = fs::OpenOptions::new()
        .write(true)
        .open(&partial)
        .and_then(|file| file.sync_all())
        .with_context(|| format!("sync imported model {}", partial.display()))
    {
        let _ = fs::remove_file(&partial);
        return Err(error);
    }
    finalize_verified_partial(models_dir, manifest, "manual")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::PARAKEET_MODEL_ID;
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;

    fn tiny_manifest(bytes: &[u8]) -> ModelArtifactManifest {
        ModelArtifactManifest {
            schema_version: 1,
            model_id: "test-model".to_string(),
            display_name: "Test".to_string(),
            description: "Test".to_string(),
            artifact_repository: "example/repo".to_string(),
            artifact_revision: "a".repeat(40),
            source_url: "https://example.invalid/model.gguf".to_string(),
            base_repository: "example/base".to_string(),
            base_revision: "b".repeat(40),
            filename: "model.gguf".to_string(),
            format: "GGUF".to_string(),
            quantization: "Q8_0".to_string(),
            size_bytes: bytes.len() as u64,
            sha256: format!("{:x}", Sha256::digest(bytes)),
            licenses: Vec::new(),
            redistribution_status: "test".to_string(),
        }
    }

    #[test]
    fn low_disk_is_rejected_with_safety_reserve() {
        let remaining = 100;
        let required = remaining + FREE_SPACE_RESERVE_BYTES;
        assert!(ensure_available_space(required - 1, remaining).is_err());
        assert!(ensure_available_space(required, remaining).is_ok());
    }

    #[test]
    fn oversized_partial_is_rejected() {
        let manifest = tiny_manifest(b"abc");
        assert!(remaining_bytes(&manifest, 4).is_err());
    }

    #[test]
    fn manual_install_verifies_and_survives_repository_restart() {
        let directory = tempfile::tempdir().expect("temp directory");
        let source = directory.path().join("source.gguf");
        fs::write(&source, b"approved bytes").expect("write source");
        let models_dir = directory.path().join("models");
        let manifest = tiny_manifest(b"approved bytes");

        let destination = install_from_local_file(&source, &models_dir, &manifest)
            .expect("install approved local model");
        assert_eq!(
            fs::read(destination).expect("read model"),
            b"approved bytes"
        );
        assert!(has_verified_receipt(&models_dir, &manifest));

        // A fresh call has no in-memory state and proves the receipt is durable.
        assert!(has_verified_receipt(&models_dir, &manifest));
    }

    #[test]
    fn corrupt_manual_install_is_deleted_without_touching_source() {
        let directory = tempfile::tempdir().expect("temp directory");
        let source = directory.path().join("source.gguf");
        fs::write(&source, b"corrupt bytes").expect("write source");
        let models_dir = directory.path().join("models");
        let manifest = tiny_manifest(b"approved bytes");

        assert!(install_from_local_file(&source, &models_dir, &manifest).is_err());
        assert_eq!(
            fs::read(&source).expect("source preserved"),
            b"corrupt bytes"
        );
        assert!(!partial_path(&models_dir, &manifest).exists());
        assert!(!manifest.destination(&models_dir).exists());
    }

    #[test]
    fn receipt_tampering_invalidates_install() {
        let directory = tempfile::tempdir().expect("temp directory");
        let models_dir = directory.path().join("models");
        fs::create_dir_all(&models_dir).expect("create models");
        let manifest = tiny_manifest(b"approved bytes");
        fs::write(manifest.destination(&models_dir), b"approved bytes").expect("write model");
        write_receipt(&models_dir, &manifest, "manual").expect("write receipt");
        assert!(has_verified_receipt(&models_dir, &manifest));

        fs::write(receipt_path(&models_dir, &manifest), b"{}").expect("tamper receipt");
        assert!(!has_verified_receipt(&models_dir, &manifest));
    }

    #[test]
    fn same_size_model_tampering_invalidates_install() {
        let directory = tempfile::tempdir().expect("temp directory");
        let models_dir = directory.path().join("models");
        fs::create_dir_all(&models_dir).expect("create models");
        let manifest = tiny_manifest(b"approved bytes");
        let destination = manifest.destination(&models_dir);
        fs::write(&destination, b"approved bytes").expect("write model");
        write_receipt(&models_dir, &manifest, "manual").expect("write receipt");
        assert!(has_verified_receipt(&models_dir, &manifest));

        fs::write(destination, b"tampered bytes").expect("tamper model");
        assert!(!has_verified_receipt(&models_dir, &manifest));
    }

    #[test]
    fn invalid_receipt_can_be_repaired_after_artifact_reverification() {
        let directory = tempfile::tempdir().expect("temp directory");
        let models_dir = directory.path().join("models");
        fs::create_dir_all(&models_dir).expect("create models");
        let manifest = tiny_manifest(b"approved bytes");
        fs::write(manifest.destination(&models_dir), b"approved bytes").expect("write model");
        fs::write(receipt_path(&models_dir, &manifest), b"invalid").expect("invalid receipt");

        verify_artifact(&manifest.destination(&models_dir), &manifest).expect("reverify model");
        write_receipt(&models_dir, &manifest, "download").expect("repair receipt");
        assert!(has_verified_receipt(&models_dir, &manifest));
    }

    #[test]
    fn production_manifest_is_reachable_by_stable_id() {
        assert!(crate::catalog::manifest_for(PARAKEET_MODEL_ID).is_some());
    }

    #[test]
    fn redirect_allowlist_rejects_unapproved_or_insecure_hosts() {
        assert!(approved_delivery_host("huggingface.co"));
        assert!(approved_delivery_host("cdn-lfs.huggingface.co"));
        assert!(approved_delivery_host("cas-bridge.xethub.hf.co"));
        assert!(!approved_delivery_host("huggingface.co.example.invalid"));
        assert!(!approved_delivery_host("example.com"));
    }

    #[test]
    #[ignore = "requires the pinned 731 MB Parakeet artifact"]
    fn live_parakeet_manual_install_creates_a_loadable_session() {
        let source = std::env::var_os("FREEFLOW_LIVE_PARAKEET_PATH")
            .map(PathBuf::from)
            .expect("set FREEFLOW_LIVE_PARAKEET_PATH to the pinned GGUF");
        let manifest = crate::catalog::manifest_for(PARAKEET_MODEL_ID).expect("manifest");
        let directory = tempfile::tempdir().expect("temporary install root");
        let models_dir = directory.path().join("models");
        let destination = install_from_local_file(&source, &models_dir, manifest)
            .expect("install pinned Parakeet artifact");
        assert!(has_verified_receipt(&models_dir, manifest));

        transcribe_cpp::init_backends_default().expect("initialize transcription backends");
        let model = transcribe_cpp::Model::load(&destination).expect("load installed model");
        let session = model.session().expect("create transcription session");
        println!("installed_path={}", destination.display());
        println!("backend={}", model.backend());
        println!("capabilities={:?}", model.capabilities());
        println!("session_created=true");
        drop(session);
    }

    #[tokio::test(flavor = "current_thread")]
    #[ignore = "downloads the pinned 731 MB Parakeet artifact after explicit license acceptance"]
    async fn live_parakeet_direct_download_creates_a_loadable_session() {
        const ACCEPTANCE: &str = "I_ACCEPT_NVIDIA_OML_AND_CC_BY_4_0";
        assert_eq!(
            std::env::var("FREEFLOW_MODEL_LICENSE_ACCEPTANCE").as_deref(),
            Ok(ACCEPTANCE),
            "set FREEFLOW_MODEL_LICENSE_ACCEPTANCE={ACCEPTANCE} only after reviewing the manifest license disclosures"
        );

        let manifest = crate::catalog::manifest_for(PARAKEET_MODEL_ID).expect("manifest");
        let accepted = crate::catalog::accepted_manifest(PARAKEET_MODEL_ID, &manifest.digest())
            .expect("bind live test to current manifest digest");
        let directory = tempfile::tempdir().expect("temporary install root");
        let models_dir = directory.path().join("models");
        let partial = partial_path(&models_dir, accepted);
        let client = approved_model_http_client().expect("restricted model HTTP client");
        let cancel = CancellationToken::new();

        loop {
            let partial_size = partial
                .metadata()
                .map(|metadata| metadata.len())
                .unwrap_or(0);
            check_disk_space(&models_dir, accepted, partial_size).expect("model disk space");
            match transfer_to_partial(
                &client,
                &accepted.source_url,
                &partial,
                accepted.size_bytes,
                &cancel,
                |_, _| {},
            )
            .await
            .expect("approved model transfer")
            {
                TransferOutcome::Complete => break,
                TransferOutcome::RestartRequired => continue,
                TransferOutcome::Cancelled => {
                    panic!("live model transfer was unexpectedly cancelled")
                }
            }
        }

        let destination = finalize_verified_partial(&models_dir, accepted, "download")
            .expect("verify and install downloaded model");
        assert!(has_verified_receipt(&models_dir, accepted));
        transcribe_cpp::init_backends_default().expect("initialize transcription backends");
        let model = transcribe_cpp::Model::load(&destination).expect("load downloaded model");
        let session = model.session().expect("create transcription session");
        println!("manifest_digest={}", accepted.digest());
        println!("installed_path={}", destination.display());
        println!("backend={}", model.backend());
        println!("capabilities={:?}", model.capabilities());
        println!("session_created=true");
        drop(session);
    }

    #[tokio::test]
    async fn offline_transfer_fails_without_creating_a_partial() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test port");
        let address = listener.local_addr().expect("test address");
        drop(listener);
        let directory = tempfile::tempdir().expect("temp directory");
        let partial = directory.path().join("model.partial");
        let result = transfer_to_partial(
            &reqwest::Client::new(),
            &format!("http://{address}/model"),
            &partial,
            8,
            &CancellationToken::new(),
            |_, _| {},
        )
        .await;
        assert!(result.is_err());
        assert!(!partial.exists());
    }

    #[tokio::test]
    async fn range_transfer_resumes_an_existing_partial() {
        let body = b"approved";
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("test address");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut request = [0u8; 2048];
            let count = stream.read(&mut request).expect("read request");
            let request = String::from_utf8_lossy(&request[..count]);
            assert!(request.contains("range: bytes=3-") || request.contains("Range: bytes=3-"));
            stream
                .write_all(b"HTTP/1.1 206 Partial Content\r\nContent-Length: 5\r\nContent-Range: bytes 3-7/8\r\nConnection: close\r\n\r\nroved")
                .expect("write response");
        });
        let directory = tempfile::tempdir().expect("temp directory");
        let partial = directory.path().join("model.partial");
        fs::write(&partial, b"app").expect("write partial");
        let outcome = transfer_to_partial(
            &reqwest::Client::new(),
            &format!("http://{address}/model"),
            &partial,
            body.len() as u64,
            &CancellationToken::new(),
            |_, _| {},
        )
        .await
        .expect("resume transfer");
        server.join().expect("server thread");
        assert_eq!(outcome, TransferOutcome::Complete);
        assert_eq!(fs::read(partial).expect("read partial"), body);
    }

    #[tokio::test]
    async fn cancellation_keeps_a_resumable_partial() {
        const EXPECTED_SIZE: usize = 128 * 1024;
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("test address");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut request = [0u8; 2048];
            let _ = stream.read(&mut request).expect("read request");
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {EXPECTED_SIZE}\r\nConnection: close\r\n\r\n"
            )
            .expect("write headers");
            stream.write_all(&vec![b'a'; 1024]).expect("first chunk");
            stream.flush().expect("flush first chunk");
            thread::sleep(Duration::from_millis(150));
            let _ = stream.write_all(&vec![b'b'; EXPECTED_SIZE - 1024]);
        });
        let directory = tempfile::tempdir().expect("temp directory");
        let partial = directory.path().join("model.partial");
        let cancel = CancellationToken::new();
        let outcome = transfer_to_partial(
            &reqwest::Client::new(),
            &format!("http://{address}/model"),
            &partial,
            EXPECTED_SIZE as u64,
            &cancel,
            |downloaded, _| {
                if downloaded > 0 {
                    cancel.cancel();
                }
            },
        )
        .await
        .expect("cancel transfer");
        server.join().expect("server thread");
        let partial_size = partial.metadata().expect("partial remains").len();
        assert_eq!(outcome, TransferOutcome::Cancelled);
        assert!(partial_size > 0);
        assert!(partial_size < EXPECTED_SIZE as u64);
    }
}
