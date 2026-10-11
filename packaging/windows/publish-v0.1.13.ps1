# One-version publication after the existing build and verified draft upload.
$ErrorActionPreference='Stop'
trap {
    $message=$_.Exception.Message.Replace($env:GH_TOKEN,'[redacted]').Replace('%','%25').Replace("`r",'%0D').Replace("`n",'%0A')
    Write-Host "::error::$message"
    exit 1
}
$version='0.1.13'
$tag="v$version"
$repository='Flwrian/FlowDPSMeter'
if($env:GITHUB_ACTIONS -ne 'true' -or !$env:GH_TOKEN -or $env:GITHUB_REPOSITORY -ne $repository -or $env:GITHUB_REF -ne "refs/tags/$tag"){throw 'Publication is restricted to the requested v0.1.13 tagged build.'}
if((Get-Content package.json -Raw | ConvertFrom-Json).version -ne $version){throw 'Application version differs.'}
$headers=@{Authorization="Bearer $env:GH_TOKEN";Accept='application/vnd.github+json';'X-GitHub-Api-Version'='2026-03-10'}
$api="https://api.github.com/repos/$repository"
$commit=Invoke-RestMethod "$api/commits/$tag" -Headers $headers
if($commit.sha -ne $env:GITHUB_SHA){throw 'Tag differs from the built commit.'}
$releases=Invoke-RestMethod "$api/releases?per_page=100" -Headers $headers
$matches=@($releases | Where-Object tag_name -eq $tag)
if($matches.Count -ne 1){throw 'Missing or duplicate release.'}
$release=$matches[0]
if($release.prerelease){throw 'Unexpected prerelease.'}
$names=@('Windows-x64','Update-Windows-x64','Sources') | ForEach-Object {"AionPartyMeter-$version-$_.zip"}
if(@($release.assets).Count -ne 3){throw 'Unexpected asset count.'}
foreach($name in $names){
    $assets=@($release.assets | Where-Object name -eq $name)
    if($assets.Count -ne 1){throw "Missing or duplicate archive: $name"}
    $asset=$assets[0]
    $file=Join-Path 'release' $name
    $digest='sha256:'+(Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
    if($asset.state -ne 'uploaded' -or $asset.size -ne (Get-Item -LiteralPath $file).Length -or $asset.digest -ne $digest){throw "Archive integrity differs: $name"}
    Write-Host "$name verified: $digest"
}
if(!$release.draft){Write-Host 'Already public; preserving release.';exit 0}
$notes=Get-Content 'docs/releases/0.1.13.md' -Raw -Encoding utf8
$body=@{name='AionPartyMeter 0.1.13 - PV de la cible';body=$notes;draft=$false;prerelease=$false;make_latest='true'} | ConvertTo-Json
$published=Invoke-RestMethod -Method Patch "$api/releases/$($release.id)" -Headers $headers -ContentType 'application/json; charset=utf-8' -Body ([System.Text.Encoding]::UTF8.GetBytes($body))
if($published.draft -or $published.prerelease -or $published.tag_name -ne $tag -or !$published.published_at){throw 'Publication not confirmed.'}
$latest=Invoke-RestMethod "$api/releases/latest" -Headers $headers
if($latest.id -ne $published.id){throw 'Release is not latest.'}
foreach($name in $names){
    $asset=@($latest.assets | Where-Object name -eq $name)[0]
    if($asset.browser_download_url -ne "https://github.com/$repository/releases/download/$tag/$name"){throw "Unexpected public download URL: $name"}
}
Write-Host "Published latest release: $($latest.html_url)"
