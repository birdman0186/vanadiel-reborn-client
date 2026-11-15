[CmdletBinding()]
param(
    [string]$GameRoot = 'E:\Final Fantasy XI\SquareEnix\FINAL FANTASY XI',
    [string]$OutputDirectory = [Environment]::GetFolderPath('Desktop')
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$source = (Resolve-Path -LiteralPath $GameRoot).ProviderPath
$output = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputDirectory)
$separator = [IO.Path]::DirectorySeparatorChar
$sourcePrefix = $source.TrimEnd($separator) + $separator
if ($output.Equals($source, [StringComparison]::OrdinalIgnoreCase) -or
    $output.StartsWith($sourcePrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Choose an output directory outside the game installation.'
}

$required = @('VTABLE.DAT', 'FTABLE.DAT')
$required += @(14..21 | ForEach-Object { 'ROM/0/{0}.DAT' -f $_ })
foreach ($relative in $required) {
    if (-not (Test-Path -LiteralPath (Join-Path $source $relative) -PathType Leaf)) {
        throw "A required UI resource is missing: $relative"
    }
}

$files = @('VTABLE.DAT', 'FTABLE.DAT', 'patch.ver')
$files += @(0..39 | ForEach-Object { 'ROM/0/{0}.DAT' -f $_ })
$files += @('ROM/119/51.DAT', 'ROM/280/15.DAT', 'ROM/324/95.DAT')
$id = [Guid]::NewGuid().ToString('N')
$stage = Join-Path ([IO.Path]::GetTempPath()) ('vdr-ui-' + $id)
$zip = Join-Path $output ('VDR-Original-UI-Resources-' + $id.Substring(0, 8) + '.zip')
$records = @()
$missing = @()

try {
    New-Item -ItemType Directory -Path $stage | Out-Null
    foreach ($relative in $files) {
        $from = Join-Path $source $relative
        if (-not (Test-Path -LiteralPath $from -PathType Leaf)) {
            $missing += $relative
            continue
        }
        $to = Join-Path $stage $relative
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $to) | Out-Null
        $before = (Get-FileHash -LiteralPath $from -Algorithm SHA256).Hash.ToLowerInvariant()
        Copy-Item -LiteralPath $from -Destination $to
        $copied = (Get-FileHash -LiteralPath $to -Algorithm SHA256).Hash.ToLowerInvariant()
        $after = (Get-FileHash -LiteralPath $from -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($before -ne $after -or $before -ne $copied) {
            throw "The file changed during collection: $relative"
        }
        $records += [ordered]@{
            path = $relative
            bytes = (Get-Item -LiteralPath $to).Length
            sha256 = $copied
        }
    }
    [ordered]@{
        schema_version = 1
        purpose = 'Original FFXI UI resource inspection'
        files = @($records)
        optional_files_missing = @($missing)
    } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $stage 'ui-resources.json') -Encoding UTF8

    New-Item -ItemType Directory -Force -Path $output | Out-Null
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip
    Write-Output $zip
} finally {
    if (Test-Path -LiteralPath $stage) {
        Remove-Item -LiteralPath $stage -Recurse -Force
    }
}
