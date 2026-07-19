param(
    [Parameter(Mandatory = $true)][string]$Archive,
    [Parameter(Mandatory = $true)][string]$Manifest,
    [Parameter(Mandatory = $true)][string]$ExtractionRoot,
    [Parameter(Mandatory = $true)][string]$OutputDirectory,
    [Parameter(Mandatory = $true)][string]$FlacExecutable
)

$ErrorActionPreference = "Stop"
$expectedArchiveSha256 = "39fde525e59672dc6d1551919b1478f724438a95aa55f874b576be21967e6c23"
$actualArchiveSha256 = (Get-FileHash -LiteralPath $Archive -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actualArchiveSha256 -ne $expectedArchiveSha256) {
    throw "LibriSpeech archive SHA-256 mismatch: expected $expectedArchiveSha256, got $actualArchiveSha256"
}

New-Item -ItemType Directory -Path $ExtractionRoot -Force | Out-Null
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$testCleanRoot = Join-Path $ExtractionRoot "LibriSpeech\test-clean"
if (-not (Test-Path -LiteralPath $testCleanRoot -PathType Container)) {
    tar -xzf $Archive -C $ExtractionRoot
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to extract $Archive"
    }
}

$corpus = Get-Content -LiteralPath $Manifest -Raw | ConvertFrom-Json
foreach ($item in $corpus.items) {
    $parts = $item.id.Split("-")
    if ($parts.Count -ne 3) {
        throw "Unexpected LibriSpeech item id: $($item.id)"
    }
    $source = Join-Path $ExtractionRoot "LibriSpeech\test-clean\$($parts[0])\$($parts[1])\$($item.id).flac"
    if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
        throw "Missing corpus source: $source"
    }
    $destination = Join-Path $OutputDirectory "$($item.id).wav"
    & $FlacExecutable --decode --force --silent --output-name=$destination $source
    if ($LASTEXITCODE -ne 0) {
        throw "FLAC conversion failed for $($item.id)"
    }
}

$wavCount = (Get-ChildItem -LiteralPath $OutputDirectory -Filter *.wav -File).Count
if ($wavCount -ne $corpus.items.Count) {
    throw "Expected $($corpus.items.Count) WAV files, found $wavCount"
}

Write-Output "Prepared $wavCount verified FF-V3 corpus WAV files."
