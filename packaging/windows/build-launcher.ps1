param([string]$OutputDirectory=(Join-Path $PSScriptRoot 'bin'),[switch]$Test)
$ErrorActionPreference='Stop'
$compiler=Join-Path $env:WINDIR 'Microsoft.NET/Framework64/v4.0.30319/csc.exe'
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$references=@('/r:System.Windows.Forms.dll','/r:System.Drawing.dll','/r:System.Web.Extensions.dll','/r:System.IO.Compression.dll','/r:System.IO.Compression.FileSystem.dll')
$sources=@((Join-Path $PSScriptRoot 'Launcher.cs'),(Join-Path $PSScriptRoot 'Updater.cs'),(Join-Path $PSScriptRoot 'UpdateStartup.cs'))
& $compiler /nologo /target:winexe /platform:x64 @references "/win32icon:$PSScriptRoot/../../src-tauri/icons/icon.ico" "/out:$OutputDirectory/DEMARRER.exe" @sources
if($LASTEXITCODE -ne 0){throw 'Compilation du lanceur échouée.'}
if($Test){
 & $compiler /nologo /target:exe /platform:x64 /main:UpdaterTests @references "/out:$OutputDirectory/updater-tests.exe" @sources (Join-Path $PSScriptRoot 'UpdaterTests.cs')
 if($LASTEXITCODE -ne 0){throw 'Compilation des tests de mise à jour échouée.'}
 & "$OutputDirectory/updater-tests.exe" $OutputDirectory
 if($LASTEXITCODE -ne 0){throw 'Tests de mise à jour échoués.'}
}
