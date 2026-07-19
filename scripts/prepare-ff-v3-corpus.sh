#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 4 ]]; then
  echo "usage: $0 ARCHIVE MANIFEST EXTRACTION_ROOT OUTPUT_DIRECTORY" >&2
  exit 2
fi

archive=$1
manifest=$2
extraction_root=$3
output_directory=$4
expected_archive_sha256=39fde525e59672dc6d1551919b1478f724438a95aa55f874b576be21967e6c23
actual_archive_sha256=$(shasum -a 256 "$archive" | awk '{print $1}')
if [[ "$actual_archive_sha256" != "$expected_archive_sha256" ]]; then
  echo "LibriSpeech archive SHA-256 mismatch: expected $expected_archive_sha256, got $actual_archive_sha256" >&2
  exit 1
fi

mkdir -p "$extraction_root" "$output_directory"
if [[ ! -d "$extraction_root/LibriSpeech/test-clean" ]]; then
  tar -xzf "$archive" -C "$extraction_root"
fi

while IFS= read -r item_id; do
  speaker=${item_id%%-*}
  remainder=${item_id#*-}
  chapter=${remainder%%-*}
  source="$extraction_root/LibriSpeech/test-clean/$speaker/$chapter/$item_id.flac"
  destination="$output_directory/$item_id.wav"
  if [[ ! -f "$source" ]]; then
    echo "Missing corpus source: $source" >&2
    exit 1
  fi
  flac --decode --force --silent --output-name="$destination" "$source"
done < <(jq -r '.items[].id' "$manifest")

expected_count=$(jq '.items | length' "$manifest")
actual_count=$(find "$output_directory" -type f -name '*.wav' | wc -l | tr -d ' ')
if [[ "$actual_count" != "$expected_count" ]]; then
  echo "Expected $expected_count WAV files, found $actual_count" >&2
  exit 1
fi

echo "Prepared $actual_count verified FF-V3 corpus WAV files."
