$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName System.IO.Compression.FileSystem
$base = Join-Path ([IO.Path]::GetTempPath()) ('vdr-collector-test-' + [Guid]::NewGuid().ToString('N'))
$game = Join-Path $base 'Game Files With Spaces'
$output = Join-Path $base 'Output'
$collector = Join-Path $PSScriptRoot 'Collect-VdrUiResources.ps1'

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

try {
    New-Item -ItemType Directory -Force -Path (Join-Path $game 'ROM/0') | Out-Null
    $required = @('VTABLE.DAT', 'FTABLE.DAT', 'ROM/0/13.DAT')
    $required += @(14..21 | ForEach-Object { 'ROM/0/{0}.DAT' -f $_ })
    foreach ($relative in $required) {
        [IO.File]::WriteAllBytes((Join-Path $game $relative), [Text.Encoding]::UTF8.GetBytes($relative))
    }
    [IO.File]::WriteAllText((Join-Path $game 'account-private.txt'), 'excluded-data')
    $before = @{}
    foreach ($file in Get-ChildItem -LiteralPath $game -File -Recurse) {
        $before[$file.FullName] = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash
    }

    $archivePath = & $collector -GameRoot $game -OutputDirectory $output
    Assert-True (Test-Path -LiteralPath $archivePath -PathType Leaf) 'The collector must create an archive.'
    $archive = [IO.Compression.ZipFile]::OpenRead($archivePath)
    try {
        $entries = @($archive.Entries | Where-Object { $_.Name } | ForEach-Object { $_.FullName.Replace('\', '/') })
        Assert-True (-not ($entries -contains 'account-private.txt')) 'Unrelated files must be excluded.'
        foreach ($relative in $required) {
            Assert-True ($entries -contains $relative) "The archive must contain $relative."
        }
        $manifestEntry = $archive.Entries | Where-Object { $_.FullName -eq 'ui-resources.json' }
        $reader = New-Object IO.StreamReader($manifestEntry.Open())
        try { $manifest = $reader.ReadToEnd() | ConvertFrom-Json } finally { $reader.Dispose() }
        Assert-True ($manifest.files.Count -eq $required.Count) 'The manifest must contain every copied file.'
        Assert-True ($manifest.optional_files_missing -contains 'ROM/119/51.DAT') 'Missing optional files must be reported.'
        foreach ($record in $manifest.files) {
            $entry = $archive.Entries | Where-Object { $_.FullName.Replace('\', '/') -eq $record.path }
            $stream = $entry.Open()
            $sha = [Security.Cryptography.SHA256]::Create()
            try {
                $actual = [BitConverter]::ToString($sha.ComputeHash($stream)).Replace('-', '').ToLowerInvariant()
                Assert-True ($actual -eq $record.sha256) 'Archive bytes must match the manifest hash.'
            } finally { $stream.Dispose(); $sha.Dispose() }
        }
    } finally { $archive.Dispose() }

    foreach ($path in $before.Keys) {
        Assert-True ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -eq $before[$path]) 'Game files must remain unchanged.'
    }
    $inside = Join-Path $game 'New Output'
    $rejected = $false
    try { & $collector -GameRoot $game -OutputDirectory $inside | Out-Null } catch { $rejected = $true }
    Assert-True $rejected 'An output directory inside the game must be rejected.'
    Assert-True (-not (Test-Path -LiteralPath $inside)) 'Rejected output must not create a game directory.'

    Remove-Item -LiteralPath (Join-Path $game 'ROM/0/14.DAT')
    $rejected = $false
    try { & $collector -GameRoot $game -OutputDirectory $output | Out-Null } catch { $rejected = $true }
    Assert-True $rejected 'Missing required window resources must be rejected.'
    Write-Output 'PASS: archive contents, hashes, source preservation, output boundary, and required resources.'
} finally {
    if (Test-Path -LiteralPath $base) { Remove-Item -LiteralPath $base -Recurse -Force }
}
