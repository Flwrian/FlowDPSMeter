param([string]$OutputDirectory='release/dependency-seed',[string]$ReusePack='')
$ErrorActionPreference='Stop'
$seed=[IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Path (Join-Path $seed 'dependances') -Force | Out-Null
$names=@('vcruntime140.dll','vcruntime140_1.dll','dependances/MicrosoftEdgeWebView2RuntimeInstallerX64.exe')
if($ReusePack){
    $previous=[IO.Path]::GetFullPath($ReusePack)
    $hashes=Get-Content -LiteralPath (Join-Path $previous 'SHA256.json') -Raw | ConvertFrom-Json
    foreach($name in $names){
        $source=Join-Path $previous $name
        if((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -ne $hashes.$name){throw "Dependency hash mismatch: $name"}
        Copy-Item -LiteralPath $source -Destination (Join-Path $seed $name) -Force
    }
}else{
    $vswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    $visualStudio=& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if(!$visualStudio){throw 'Visual Studio C++ redist files not found.'}
    $redist=Join-Path $visualStudio 'VC/Redist/MSVC'
    $runtime=Get-ChildItem -LiteralPath $redist -Directory | Sort-Object Name -Descending | ForEach-Object {Join-Path $_.FullName 'x64/Microsoft.VC143.CRT'} | Where-Object {Test-Path -LiteralPath (Join-Path $_ 'vcruntime140.dll')} | Select-Object -First 1
    if(!$runtime){throw 'Microsoft x64 runtime DLLs not found.'}
    foreach($name in $names[0..1]){Copy-Item -LiteralPath (Join-Path $runtime $name) -Destination (Join-Path $seed $name)}
    Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/?LinkId=2124701' -OutFile (Join-Path $seed $names[2])
}
$manifest=@{}
foreach($name in $names){
    $path=Join-Path $seed $name
    $signature=Get-AuthenticodeSignature -LiteralPath $path
    if($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation'){throw "Invalid Microsoft signature: $name"}
    $manifest[$name]=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
}
$manifest | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $seed 'SHA256.json') -Encoding UTF8
Write-Host 'Microsoft dependency signatures and hashes verified. No installer executed.'
