# ClumsyCat Windows Compatibility Test Script
# Tests build, binary verification, config paths, tool detection, and Windows-specific behavior

param(
    [switch]$Verbose
)

$ErrorActionPreference = "Continue"
$script:PassCount = 0
$script:FailCount = 0
$script:WarnCount = 0

function Write-TestHeader {
    param([string]$Message)
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host $Message -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
}

function Write-TestPass {
    param([string]$Message)
    Write-Host "[PASS] $Message" -ForegroundColor Green
    $script:PassCount++
}

function Write-TestFail {
    param([string]$Message)
    Write-Host "[FAIL] $Message" -ForegroundColor Red
    $script:FailCount++
}

function Write-TestWarn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
    $script:WarnCount++
}

function Write-TestInfo {
    param([string]$Message)
    if ($Verbose) {
        Write-Host "[INFO] $Message" -ForegroundColor Gray
    }
}

# Test 1: Build Project
Write-TestHeader "Test 1: Building ClumsyCat with Cargo"
try {
    Write-TestInfo "Running: cargo build --release"
    $buildOutput = cargo build --release 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-TestPass "Cargo build completed successfully"
    } else {
        Write-TestFail "Cargo build failed with exit code $LASTEXITCODE"
        if ($Verbose) {
            Write-Host $buildOutput
        }
    }
} catch {
    Write-TestFail "Cargo build exception: $_"
}

# Test 2: Verify Binary Exists
Write-TestHeader "Test 2: Verifying Binary Exists"
$binaryPath = "target\release\clumsycat.exe"
if (Test-Path $binaryPath) {
    Write-TestPass "Binary exists at: $binaryPath"
    $fileInfo = Get-Item $binaryPath
    Write-TestInfo "Binary size: $($fileInfo.Length) bytes"
    Write-TestInfo "Last modified: $($fileInfo.LastWriteTime)"
} else {
    Write-TestFail "Binary not found at: $binaryPath"
}

# Test 3: Config Directory Creation
Write-TestHeader "Test 3: Testing Config Directory Creation"
$configDir = Join-Path $env:APPDATA "clumsycat"
$configFile = Join-Path $configDir "config.json"

Write-TestInfo "Expected config location: $configFile"

if (Test-Path $configDir) {
    Write-TestPass "Config directory exists: $configDir"

    if (Test-Path $configFile) {
        Write-TestPass "Config file exists: $configFile"
        try {
            $configContent = Get-Content $configFile -Raw | ConvertFrom-Json
            Write-TestInfo "Config has settings: $($configContent.settings -ne $null)"
            Write-TestInfo "Config has favorites: $($configContent.favorites -ne $null)"
            Write-TestInfo "Config has recents: $($configContent.recents -ne $null)"
            Write-TestPass "Config file is valid JSON"
        } catch {
            Write-TestWarn "Config file exists but may be invalid JSON: $_"
        }
    } else {
        Write-TestWarn "Config file does not exist yet (will be created on first run)"
    }
} else {
    Write-TestWarn "Config directory does not exist yet (will be created on first run)"
}

# Test 4: Check for Installed AI Tools
Write-TestHeader "Test 4: Checking for Installed AI Tools"
$tools = @{
    "claude" = "Claude Code"
    "codex" = "Codex"
    "kilo" = "Kilocode"
    "gemini" = "Gemini Code Assist"
    "opencode" = "OpenCode"
}

$foundTools = 0
foreach ($cmd in $tools.Keys) {
    $toolName = $tools[$cmd]
    try {
        $null = Get-Command $cmd -ErrorAction SilentlyContinue
        if ($?) {
            Write-TestPass "$toolName found in PATH ($cmd)"
            $foundTools++
        } else {
            Write-TestInfo "$toolName not found ($cmd)"
        }
    } catch {
        Write-TestInfo "$toolName not found ($cmd)"
    }
}

if ($foundTools -eq 0) {
    Write-TestWarn "No AI coding tools found in PATH"
} else {
    Write-TestPass "Found $foundTools AI coding tool(s) in PATH"
}

# Test 5: Verify ASCII Art File
Write-TestHeader "Test 5: Verifying ASCII Art File"
$asciiFiles = @("ascii_cat.md", "ascii.md")
$asciiFound = $false

foreach ($file in $asciiFiles) {
    if (Test-Path $file) {
        Write-TestPass "ASCII art file found: $file"
        $lineCount = (Get-Content $file).Count
        Write-TestInfo "File contains $lineCount lines"
        $asciiFound = $true
        break
    }
}

if (-not $asciiFound) {
    Write-TestWarn "ASCII art file not found (checked: $($asciiFiles -join ', '))"
}

# Test 6: Basic Version Check
Write-TestHeader "Test 6: Running Version Check"
if (Test-Path $binaryPath) {
    try {
        Write-TestInfo "Running: $binaryPath --version"
        $versionOutput = & $binaryPath --version 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-TestPass "Version check successful: $versionOutput"
        } else {
            Write-TestFail "Version check failed with exit code $LASTEXITCODE"
        }
    } catch {
        Write-TestWarn "Version check exception: $_"
    }
} else {
    Write-TestFail "Cannot run version check - binary not found"
}

# Test 7: Terminal ANSI Support Detection
Write-TestHeader "Test 7: Testing Terminal ANSI Support"
$ansiSupported = $false

if ($env:WT_SESSION) {
    Write-TestPass "Windows Terminal detected (WT_SESSION env var set)"
    $ansiSupported = $true
}

if ([Environment]::OSVersion.Version.Build -ge 10586) {
    Write-TestPass "Windows 10+ detected (build $([Environment]::OSVersion.Version.Build)) - ANSI support available"
    $ansiSupported = $true
}

try {
    $vtEnabled = [Console]::OutputEncoding -eq [System.Text.Encoding]::UTF8
    if ($vtEnabled) {
        Write-TestPass "Console UTF-8 encoding enabled"
    } else {
        Write-TestInfo "Console encoding: $([Console]::OutputEncoding.EncodingName)"
    }
} catch {
    Write-TestInfo "Could not detect console encoding"
}

if (-not $ansiSupported) {
    Write-TestWarn "ANSI support may be limited - recommend using Windows Terminal"
}

# Test 8: PATH Separator Handling
Write-TestHeader "Test 8: Checking PATH Separator Handling"
$pathSeparator = [System.IO.Path]::PathSeparator
Write-TestInfo "System PATH separator: '$pathSeparator' (should be ';' on Windows)"

if ($pathSeparator -eq ';') {
    Write-TestPass "Correct Windows PATH separator detected"
} else {
    Write-TestFail "Unexpected PATH separator: '$pathSeparator'"
}

$envPath = $env:PATH
$pathEntries = $envPath -split $pathSeparator
Write-TestInfo "PATH contains $($pathEntries.Count) entries"

$cargoInPath = $pathEntries | Where-Object { $_ -like "*\.cargo\bin*" }
if ($cargoInPath) {
    Write-TestPass "Cargo bin directory found in PATH: $cargoInPath"
} else {
    Write-TestWarn "Cargo bin directory not found in PATH"
}

# Test 9: Cargo.toml Verification
Write-TestHeader "Test 9: Verifying Cargo.toml Configuration"
if (Test-Path "Cargo.toml") {
    $cargoContent = Get-Content "Cargo.toml" -Raw

    if ($cargoContent -match 'name\s*=\s*"clumsycat"') {
        Write-TestPass "Binary name 'clumsycat' found in Cargo.toml"
    } else {
        Write-TestFail "Binary name not correctly configured in Cargo.toml"
    }

    if ($cargoContent -match "windows-sys") {
        Write-TestPass "Windows-specific dependencies found in Cargo.toml"
    } else {
        Write-TestWarn "Windows-sys dependency not found in Cargo.toml"
    }

    if ($cargoContent -match 'version\s*=\s*"([^"]+)"') {
        $version = $matches[1]
        Write-TestPass "Package version: $version"
    }
} else {
    Write-TestFail "Cargo.toml not found"
}

# Test 10: Clippy Lint Check
Write-TestHeader "Test 10: Running Clippy Lint Check"
try {
    Write-TestInfo "Running: cargo clippy -- -D warnings"
    $clippyOutput = cargo clippy -- -D warnings 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-TestPass "Clippy passed with zero warnings"
    } else {
        Write-TestFail "Clippy failed with exit code $LASTEXITCODE"
        if ($Verbose) {
            Write-Host $clippyOutput
        }
    }
} catch {
    Write-TestWarn "Clippy check exception: $_"
}

# Summary
Write-Host "`n" -NoNewline
Write-TestHeader "Test Summary"
Write-Host "Total Passed: " -NoNewline
Write-Host $script:PassCount -ForegroundColor Green
Write-Host "Total Failed: " -NoNewline
Write-Host $script:FailCount -ForegroundColor Red
Write-Host "Total Warnings: " -NoNewline
Write-Host $script:WarnCount -ForegroundColor Yellow

$totalTests = $script:PassCount + $script:FailCount
if ($totalTests -gt 0) {
    $passRate = [math]::Round(($script:PassCount / $totalTests) * 100, 2)
    Write-Host "`nPass Rate: $passRate%" -ForegroundColor $(if ($passRate -ge 80) { "Green" } elseif ($passRate -ge 60) { "Yellow" } else { "Red" })
}

Write-Host "`n========================================`n" -ForegroundColor Cyan

if ($script:FailCount -gt 0) {
    exit 1
} else {
    exit 0
}
