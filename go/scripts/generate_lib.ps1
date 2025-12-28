param (
    [string]$TargetDir
)

if ($TargetDir) {
    Set-Location $TargetDir
}

Write-Host "🔧 Generating import library in $pwd..." -ForegroundColor Cyan

$defContent = @'
LIBRARY whatsmeow
EXPORTS
    wm_client_new
    wm_client_connect
    wm_client_disconnect
    wm_client_destroy
    wm_poll_event
    wm_send_message
    wm_last_error
'@

$defContent | Out-File -FilePath whatsmeow.def -Encoding ASCII

$vsWhere = "${Env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsPath = & $vsWhere -latest -property installationPath
    $vcVars64 = "$vsPath\VC\Auxiliary\Build\vcvars64.bat"
    
    if (Test-Path $vcVars64) {
        Write-Host "🔨 Found Visual Studio vcvars64 at $vcVars64" -ForegroundColor Gray
        cmd /c "`"$vcVars64`" && lib /def:whatsmeow.def /out:whatsmeow.lib /machine:x64"
    } else {
        Write-Error "Could not find vcvars64.bat"
    }
} else {
    Write-Error "Could not find vswhere.exe"
}
