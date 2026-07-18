# Building FreeFlow

## Prerequisites

- Rust stable
- Bun 1.2.20
- Tauri 2 platform prerequisites
- Windows: Visual Studio C++ Build Tools and Vulkan SDK 1.4 or newer
- macOS: current Xcode command-line tools

## Developer build

```text
bun install --frozen-lockfile
bun run lint
bun run format:check
bun run build
cargo test --manifest-path src-tauri/Cargo.toml
bun run tauri build --no-bundle
```

On Windows, open a Visual Studio developer shell and ensure `VULKAN_SDK` points
to the installed SDK. On macOS, grant microphone and Accessibility permission
only to a locally built app you trust.

No model is downloaded during the build. Put approved local evaluation weights
outside Git and select them manually. Release signing and automatic updates are
disabled until FreeFlow owns and documents those channels.
