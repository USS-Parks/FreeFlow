use std::path::PathBuf;
use transcribe_cpp::{init_backends_default, Backend, Model, ModelOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: verify_model_load <model.gguf>")?;

    init_backends_default()?;
    let model = Model::load_with(
        &model_path,
        &ModelOptions {
            backend: Backend::Auto,
            ..Default::default()
        },
    )?;
    let capabilities = model.capabilities();
    let _session = model.session()?;

    println!("model_path={}", model_path.display());
    println!("backend={}", model.backend());
    println!("capabilities={capabilities:?}");
    println!("session_created=true");
    Ok(())
}
