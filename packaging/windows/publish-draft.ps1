# Runs only on GitHub Actions, using the repository-scoped workflow token.
param([string]$AssetDirectory='release')
$ErrorActionPreference='Stop'
if($env:GITHUB_ACTIONS -ne 'true' -or !$env:GH_TOKEN){throw 'This script requires GitHub Actions and its repository token.'}
$repository=$env:GITHUB_REPOSITORY
if($repository -notmatch '^[A-Za-z0-9-]+/[A-Za-z0-9_.-]+$'){throw 'Invalid repository.'}
$version=(Get-Content package.json -Raw | ConvertFrom-Json).version
if($version -notmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'){throw 'Invalid stable version.'}
$tag="v$version"
if($env:GITHUB_REF_TYPE -eq 'tag' -and $env:GITHUB_REF_NAME -ne $tag){throw 'Tag differs from the application version.'}
$headers=@{Authorization="Bearer $env:GH_TOKEN";Accept='application/vnd.github+json';'X-GitHub-Api-Version'='2026-03-10'}
$api="https://api.github.com/repos/$repository"
$release=$null
try{$release=Invoke-RestMethod -Uri "$api/releases/tags/$tag" -Headers $headers}catch{if([int]$_.Exception.Response.StatusCode -ne 404){throw}}
if(!$release){
    $body=@{tag_name=$tag;target_commitish=$env:GITHUB_SHA;name="AionPartyMeter $version";draft=$true;prerelease=$false;body="Mises a jour GitHub au lancement via DEMARRER.exe. Installer le ZIP Windows complet une premiere fois ; le ZIP Update sert au lanceur. Les sources GPL correspondantes sont jointes. Consultez le guide d'installation et de publication du depot."} | ConvertTo-Json
    $release=Invoke-RestMethod -Method Post -Uri "$api/releases" -Headers $headers -ContentType 'application/json' -Body $body
}
if(!$release.draft){throw 'The release is already public; refusing to modify its assets.'}
foreach($suffix in @('Windows-x64','Update-Windows-x64','Sources')){
    $name="AionPartyMeter-$version-$suffix.zip"
    $path=Join-Path $AssetDirectory $name
    if(!(Test-Path -LiteralPath $path -PathType Leaf)){throw "Missing release asset: $name"}
    $existing=@($release.assets | Where-Object name -eq $name)
    if($existing.Count){
        $digest='sha256:'+(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        if($existing[0].digest -ne $digest){throw "Draft already contains a different $name; review it before retrying."}
        continue
    }
    $upload="https://uploads.github.com/repos/$repository/releases/$($release.id)/assets?name=$([Uri]::EscapeDataString($name))"
    $asset=Invoke-RestMethod -Method Post -Uri $upload -Headers $headers -ContentType 'application/zip' -InFile $path
    $digest='sha256:'+(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    if($asset.digest -ne $digest){throw "Uploaded digest differs: $name"}
}
Write-Host "Draft release prepared: $($release.html_url)"
