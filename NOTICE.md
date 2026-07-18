# FreeFlow notices

## Handy foundation

FreeFlow incorporates and modifies source code from Handy:

- Repository: https://github.com/cjpais/Handy
- Imported commit: `d861e24bc825c699ccf7215a430684c6e322131c`
- Imported tree: `5b91bf5be354d7fa6f267a4f47045db0cfc718bc`
- License: MIT
- Copyright: Copyright (c) 2025 CJ Pais

The complete retained MIT notice is in `LICENSE`. FreeFlow's name, product
identity, behavior specifications, and post-import changes are independent and
are not endorsed by the Handy authors.

Dependency and model licenses remain their respective owners' licenses. ASR
weights are not distributed from this repository; the small Silero VAD model
below is the sole bundled inference weight.

## Bundled Silero VAD V4 model

FreeFlow bundles `src-tauri/resources/models/silero_vad_v4.onnx` for local
voice-activity detection.

- Upstream project: https://github.com/snakers4/silero-vad
- Vendored source: `cjpais/vad-rs` commit
  `2a412ed858695b9251f3f5a1a20d95b59fa7c498`, path
  `tests/fixtures/silero_vad_v4.onnx`
- Size: 1,807,522 bytes
- Git blob: `e6db48d6e2a0797a2ec173c008384f7710189344`
- SHA-256: `a35ebf52fd3ce5f1469b2a36158dba761bc47b973ea3382b3186ca15b1f5af28`
- License: MIT

The machine-readable provenance record is
`src-tauri/resources/models/silero-vad-v4.json`.

```text
MIT License

Copyright (c) 2020-present Silero Team

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Optional Parakeet model

FreeFlow can install, but does not bundle or redistribute, the pinned
`parakeet-unified-en-0.6b-Q8_0.gguf` artifact described in
`models/manifests/parakeet-unified-en-0.6b-q8_0.json`.

- Base model: NVIDIA `parakeet-unified-en-0.6b`, governed by the NVIDIA Open
  Model License Agreement.
- Required attribution: Licensed by NVIDIA Corporation under the NVIDIA Open
  Model License.
- GGUF conversion repository declaration: Creative Commons Attribution 4.0
  International (`CC-BY-4.0`).
- Conversion attribution: published by handy-computer and mirrored by
  Memoravox; base model by NVIDIA.

FreeFlow presents these separate license scopes and their source URLs before a
user can approve either a direct transfer or a verified manual installation.

## FF-V3 LibriSpeech evaluation references

FreeFlow retains an evaluation manifest and machine-readable result for a
20-speaker subset of the LibriSpeech `test-clean` corpus. Audio is not committed.

- Resource: OpenSLR SLR12, LibriSpeech ASR corpus
- Source: https://www.openslr.org/12
- Corpus authors: Vassil Panayotov, Guoguo Chen, Daniel Povey, and Sanjeev
  Khudanpur
- Source material: LibriVox public-domain audiobooks aligned to Project
  Gutenberg texts
- License: Creative Commons Attribution 4.0 International (`CC-BY-4.0`)
- Verified archive MD5: `32fa31d27d2e1cad72775fee3f4849a9`
- Verified archive SHA-256:
  `39fde525e59672dc6d1551919b1478f724438a95aa55f874b576be21967e6c23`

The corpus is used only for independent local-ASR measurement and does not
originate from Wispr Flow or any proprietary observation.
