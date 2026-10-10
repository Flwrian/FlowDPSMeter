"""Create reproducible release assets. No uploads, credentials or game data."""
from pathlib import Path
import argparse,hashlib,json,os,re,shutil,subprocess,zipfile

parser=argparse.ArgumentParser()
parser.add_argument('--repository',default=None)
parser.add_argument('--output',type=Path,default=Path('release'))
parser.add_argument('--launcher',type=Path,default=Path('packaging/windows/bin/DEMARRER.exe'))
parser.add_argument('--full-pack',type=Path,help='Existing portable pack to reuse verified Microsoft dependencies')
args=parser.parse_args()
root=Path(__file__).resolve().parents[2]
version=json.loads((root/'package.json').read_text())['version']
assert re.fullmatch(r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)',version)
repository=(args.repository if args.repository is not None else json.loads((root/'update.json').read_text(encoding='utf-8'))['repository']).removeprefix('https://github.com/').removesuffix('/').removesuffix('.git')
if repository and not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9-]{0,38}/[A-Za-z0-9_.-]{1,100}',repository):raise ValueError('Repository must be owner/name')
meter=root/'src-tauri/target/release/AionPartyMeter.exe'
launcher=args.launcher.resolve()
assert meter.is_file() and launcher.is_file()
out=args.output.resolve()
if root in out.parents and out.relative_to(root).parts[0]!='release':raise ValueError('Output inside the source tree must be under release/')
out.mkdir(parents=True,exist_ok=True)
payload=out/f'payload-{version}';payload.mkdir(exist_ok=False)
shutil.copy2(meter,payload/'AionPartyMeter.exe');shutil.copy2(launcher,payload/'DEMARRER.exe')
for name in ['ANALYSE.md','VALIDATION.md']:shutil.copy2(root/'docs'/name,payload/name)
guide=(root/'docs/INSTALLATION.md').read_text(encoding='utf-8')
(payload/'LIRE-MOI.txt').write_text(guide,encoding='utf-8-sig')
import html
parts=[]
for line in guide.splitlines():
    text=html.escape(line)
    text=re.sub(r'\[([^\]]+)\]\((https://[^)]+)\)',r'<a href="\2">\1</a>',text)
    text=re.sub(r'\*\*(.+?)\*\*',r'<strong>\1</strong>',text)
    parts.append('<h2>'+text[3:]+'</h2>' if line.startswith('## ') else '<h1>'+text[2:]+'</h1>' if line.startswith('# ') else '<p>'+text+'</p>')
(payload/'LIRE-MOI.html').write_text('<!doctype html><html lang="fr"><meta charset="utf-8"><title>AionPartyMeter</title><style>body{font:16px/1.65 Segoe UI,sans-serif;background:#101923;color:#dce6ef;max-width:900px;margin:40px auto;padding:0 24px}h1,h2,a{color:#81d9ce}</style>'+''.join(parts),encoding='utf-8')
(payload/'licences').mkdir(exist_ok=True)
shutil.copy2(root/'LICENSE',payload/'licences/GPL-3.0.txt')
for font in ['pretendard','noto-sans-sc','noto-sans-tc']:shutil.copy2(root/'public/vendor/fonts'/font/'LICENSE',payload/'licences'/f'{font}-OFL.txt')
(payload/'licences/DEPENDANCES.txt').write_text('Moteur A2Tools (taengu), GPL-3.0-only, commit 82e53c1008ac4c2974446cc703703f473bbbed81.\nMicrosoft Visual C++ et WebView2 : redistribution officielle non modifiée, licences Microsoft.\nhttps://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution\nhttps://www.microsoft.com/en-us/legal/terms-of-use\nhttps://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files\nhttps://aka.ms/VCRedistLicense\nNpcap : non redistribué, téléchargement officiel avec accord de licence par l’utilisateur.\nhttps://npcap.com/oem/redist\nSources correspondantes : AionPartyMeter-'+version+'-Sources.zip\n',encoding='utf-8')
# Optional full pack: dependencies come from an existing verified distribution.
if args.full_pack:
    previous=args.full_pack.resolve()
    previous_hashes=json.loads((previous/'SHA256.json').read_text(encoding='utf-8-sig'))
    for name in ['vcruntime140.dll','vcruntime140_1.dll','dependances/MicrosoftEdgeWebView2RuntimeInstallerX64.exe']:
        source=previous/name
        assert hashlib.sha256(source.read_bytes()).hexdigest()==previous_hashes[name],f'Altered dependency: {name}'
    for name in ['vcruntime140.dll','vcruntime140_1.dll']:shutil.copy2(previous/name,payload/name)

def hashes(folder):return {p.relative_to(folder).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(folder.rglob('*')) if p.is_file() and p.name not in ['SHA256.json','update-manifest.json']}
(payload/'SHA256.json').write_text(json.dumps(hashes(payload),indent=2),encoding='utf-8')
files=hashes(payload);files['SHA256.json']=hashlib.sha256((payload/'SHA256.json').read_bytes()).hexdigest()
(payload/'update-manifest.json').write_text(json.dumps(dict(schemaVersion=1,version=version,files=files),indent=2),encoding='utf-8')
update=out/f'AionPartyMeter-{version}-Update-Windows-x64.zip'
with zipfile.ZipFile(update,'w',zipfile.ZIP_DEFLATED) as archive:
    for p in sorted(payload.rglob('*')):
        if p.is_file():archive.write(p,p.relative_to(payload).as_posix())
if args.full_pack:
    pack=out/f'AionPartyMeter-{version}';pack.mkdir(exist_ok=False)
    for p in payload.rglob('*'):
        if p.is_file() and p.name!='update-manifest.json':
            dest=pack/p.relative_to(payload);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,dest)
    (pack/'dependances').mkdir(exist_ok=True)
    shutil.copy2(previous/'dependances/MicrosoftEdgeWebView2RuntimeInstallerX64.exe',pack/'dependances/MicrosoftEdgeWebView2RuntimeInstallerX64.exe')
    (pack/'exemples').mkdir(exist_ok=True)
    for p in (root/'examples').iterdir():shutil.copy2(p,pack/'exemples'/p.name)
    (pack/'update.json').write_text(json.dumps(dict(enabled=True,repository=repository),indent=2),encoding='utf-8')
    (pack/'SHA256.json').write_text(json.dumps(hashes(pack),indent=2),encoding='utf-8')
    with zipfile.ZipFile(out/f'AionPartyMeter-{version}-Windows-x64.zip','w',zipfile.ZIP_DEFLATED) as archive:
        for p in sorted(pack.rglob('*')):
            if p.is_file():archive.write(p,'AionPartyMeter/'+p.relative_to(pack).as_posix())

sources=out/f'AionPartyMeter-{version}-Sources.zip'
excluded={'.git','node_modules','dist','target','gen','bin','release'}
with zipfile.ZipFile(sources,'w',zipfile.ZIP_DEFLATED) as archive:
    for folder,dirs,names in os.walk(root):
        dirs[:]=[d for d in dirs if d not in excluded and not d.startswith('.apm-')]
        for name in names:
            p=Path(folder)/name;rel=p.relative_to(root)
            if name.endswith(('.log','.local','.apm-new')) or name.startswith('.apm-update-'):continue
            if name.startswith('.env') or name.endswith(('.pem','.key','.pfx')):continue
            if name.endswith('.apm.jsonl') and rel.parts[0]!='examples':continue
            archive.write(p,'AionPartyMeter/'+rel.as_posix())
for artifact in [update,sources]:
    with zipfile.ZipFile(artifact) as archive:assert archive.testzip() is None
    print(artifact.name,artifact.stat().st_size,'SHA256',hashlib.sha256(artifact.read_bytes()).hexdigest())
