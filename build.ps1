param([string]$Action = "check")

$ErrorActionPreference = "Continue"
$vsPath = "D:\VisualStudio"
$vcvars = "$vsPath\VC\Auxiliary\Build\vcvars64.bat"

# Source VS environment
$vsEnv = cmd /c "call `"$vcvars`" >nul 2>nul && set" 2>$null
foreach ($line in $vsEnv) {
    if ($line -match "^(.*?)=(.*)$") {
        [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
    }
}

# Add cargo
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

Set-Location "D:\OpenCode\ys"
Write-Host "Running: cargo $Action" -ForegroundColor Cyan
cargo $Action 2>&1
exit $LASTEXITCODE
