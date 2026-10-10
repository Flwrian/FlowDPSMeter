// GPL-3.0-only. GitHub Releases updater for the portable Windows pack.
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Net;
using System.Security.Cryptography;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading;
using System.Web.Script.Serialization;

internal sealed class UpdateConfig { public bool enabled=true; public string repository=""; }
internal sealed class ReleaseInfo { public string tag,version,url,digest,asset; public long size; }
internal sealed class UpdateManifest { public int schemaVersion {get;set;} public string version {get;set;} public Dictionary<string,string> files {get;set;} }
internal sealed class Journal { public string phase,backup; public Dictionary<string,bool> files; }

internal static class UpdateCore
{
    internal const long MaxZip=64*1024*1024,MaxExpanded=128*1024*1024;
    internal static readonly JavaScriptSerializer Json=new JavaScriptSerializer(){MaxJsonLength=2*1024*1024};
    internal static string Repository(string value)
    {
        value=(value??"").Trim().TrimEnd('/');
        if(value.StartsWith("https://github.com/",StringComparison.OrdinalIgnoreCase))value=value.Substring(19);
        if(value.EndsWith(".git",StringComparison.OrdinalIgnoreCase))value=value.Substring(0,value.Length-4);
        if(!Regex.IsMatch(value,@"^[A-Za-z0-9][A-Za-z0-9-]{0,38}/[A-Za-z0-9_.-]{1,100}$")||value.Split('/')[1]=="."||value.Split('/')[1]=="..")throw new InvalidDataException("Indique un dépôt GitHub sous la forme compte/AionPartyMeter.");
        return value;
    }
    internal static UpdateConfig Config(string root)
    {
        string path=Path.Combine(root,"update.json");
        if(!File.Exists(path))return new UpdateConfig();
        var config=Json.Deserialize<UpdateConfig>(File.ReadAllText(path));
        if(config==null)throw new InvalidDataException("Configuration de mise à jour invalide.");
        if(!String.IsNullOrWhiteSpace(config.repository))config.repository=Repository(config.repository);
        return config;
    }
    internal static Version VersionOf(string value)
    {
        if(!Regex.IsMatch(value??"",@"^v?(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$"))throw new InvalidDataException("Tag de version stable attendu : v0.1.8.");
        string text=value.TrimStart('v');var version=new Version(text);return new Version(version.Major,version.Minor,version.Build);
    }
    internal static Version Installed(string root)
    {
        var version=new Version(FileVersionInfo.GetVersionInfo(Path.Combine(root,"AionPartyMeter.exe")).FileVersion);
        return new Version(version.Major,version.Minor,version.Build);
    }
    private static string Str(Dictionary<string,object> obj,string key){object value;return obj.TryGetValue(key,out value)?value as string:null;}
    internal static ReleaseInfo SelectRelease(string text,string repository,Version installed)
    {
        repository=Repository(repository);var data=Json.Deserialize<Dictionary<string,object>>(text);
        if(data==null||!data.ContainsKey("draft")||!data.ContainsKey("prerelease")||!(data["draft"] is bool)||!(data["prerelease"] is bool))throw new InvalidDataException("Réponse GitHub invalide.");
        if((bool)data["draft"]||(bool)data["prerelease"])return null;
        string tag=Str(data,"tag_name");var version=VersionOf(tag);if(version<=installed)return null;
        string asset="AionPartyMeter-"+version+"-Update-Windows-x64.zip";
        object assets; if(!data.TryGetValue("assets",out assets)||!(assets is System.Collections.IEnumerable)||assets is string)throw new InvalidDataException("Liste de fichiers GitHub absente.");
        var matches=((System.Collections.IEnumerable)assets).Cast<object>().OfType<Dictionary<string,object>>().Where(a=>Str(a,"name")==asset).ToArray();
        if(matches.Length!=1)throw new InvalidDataException("La release ne contient pas le ZIP de mise à jour attendu : "+asset);
        var match=matches[0];string digest=Str(match,"digest"),url=Str(match,"browser_download_url");
        if(!Regex.IsMatch(digest??"",@"^sha256:[a-fA-F0-9]{64}$"))throw new InvalidDataException("Empreinte SHA256 GitHub absente : mise à jour refusée.");
        string expected="https://github.com/"+repository+"/releases/download/"+Uri.EscapeDataString(tag)+"/"+asset;
        if(!String.Equals(url,expected,StringComparison.OrdinalIgnoreCase))throw new InvalidDataException("Le fichier de mise à jour ne provient pas du dépôt configuré.");
        long size=Convert.ToInt64(match["size"]);if(size<=0||size>MaxZip)throw new InvalidDataException("ZIP de mise à jour trop volumineux.");
        return new ReleaseInfo(){tag=tag,version=version.ToString(),asset=asset,url=url,size=size,digest=digest.Substring(7).ToLowerInvariant()};
    }
    internal static string Hash(string path){using(var sha=SHA256.Create())using(var stream=File.OpenRead(path))return BitConverter.ToString(sha.ComputeHash(stream)).Replace("-","").ToLowerInvariant();}
    internal static bool Allowed(string name)
    {
        return new[]{"AionPartyMeter.exe","DEMARRER.exe","vcruntime140.dll","vcruntime140_1.dll","LIRE-MOI.html","LIRE-MOI.txt","ANALYSE.md","VALIDATION.md","SHA256.json"}.Contains(name,StringComparer.Ordinal)
            ||Regex.IsMatch(name,@"^licences/[A-Za-z0-9_.-]+\.(txt|md)$");
    }
    internal static string Destination(string root,string name)
    {
        if(!Allowed(name))throw new InvalidDataException("Fichier non autorisé dans la mise à jour : "+name);
        string absolute=Path.GetFullPath(Path.Combine(root,name.Replace('/',Path.DirectorySeparatorChar)));
        string parent=Path.GetFullPath(root).TrimEnd(Path.DirectorySeparatorChar)+Path.DirectorySeparatorChar;
        if(!absolute.StartsWith(parent,StringComparison.OrdinalIgnoreCase))throw new InvalidDataException("Chemin extérieur au pack.");
        for(string cursor=Path.GetDirectoryName(absolute);cursor!=null&&cursor.StartsWith(parent,StringComparison.OrdinalIgnoreCase);cursor=Path.GetDirectoryName(cursor))
            if(Directory.Exists(cursor)&&(File.GetAttributes(cursor)&FileAttributes.ReparsePoint)!=0)throw new InvalidDataException("Lien de dossier non autorisé dans le pack.");
        if(File.Exists(absolute)&&(File.GetAttributes(absolute)&FileAttributes.ReparsePoint)!=0)throw new InvalidDataException("Lien de fichier non autorisé dans le pack.");
        return absolute;
    }
    internal static string ManagedDirectory(string root,string name,string prefix)
    {
        if(!Regex.IsMatch(name,"^"+Regex.Escape(prefix)+"[a-f0-9]{32}$"))throw new InvalidDataException("Dossier temporaire invalide.");
        string path=Path.GetFullPath(Path.Combine(root,name));
        if(Path.GetDirectoryName(path)!=Path.GetFullPath(root).TrimEnd(Path.DirectorySeparatorChar))throw new InvalidDataException("Dossier temporaire extérieur au pack.");
        if(Directory.Exists(path)&&(File.GetAttributes(path)&FileAttributes.ReparsePoint)!=0)throw new InvalidDataException("Lien temporaire interdit.");
        return path;
    }
    internal static void DeleteManaged(string root,string name,string prefix)
    {
        string path=ManagedDirectory(root,name,prefix);
        if(!Directory.Exists(path))return;
        // Check all resolved targets before recursive deletion; never follow junctions.
        foreach(string child in Directory.GetFileSystemEntries(path,"*",SearchOption.AllDirectories))
            if((File.GetAttributes(child)&FileAttributes.ReparsePoint)!=0)throw new InvalidDataException("Lien temporaire interdit.");
        Directory.Delete(path,true);
    }
    internal static UpdateManifest Manifest(string stage,string expectedVersion)
    {
        string manifestPath=Path.Combine(stage,"update-manifest.json");
        if(!File.Exists(manifestPath)||new FileInfo(manifestPath).Length>128*1024)throw new InvalidDataException("Manifeste de mise à jour absent ou trop volumineux.");
        var manifest=Json.Deserialize<UpdateManifest>(File.ReadAllText(manifestPath));
        if(manifest==null||manifest.schemaVersion!=1||manifest.version!=expectedVersion||manifest.files==null||manifest.files.Count>50)throw new InvalidDataException("Manifeste de mise à jour incompatible.");
        foreach(string required in new[]{"AionPartyMeter.exe","DEMARRER.exe","SHA256.json"})if(!manifest.files.ContainsKey(required))throw new InvalidDataException("Fichier requis absent : "+required);
        foreach(var item in manifest.files){string path=Destination(stage,item.Key);if(!Regex.IsMatch(item.Value??"",@"^[a-f0-9]{64}$")||!File.Exists(path)||Hash(path)!=item.Value)throw new InvalidDataException("Intégrité invalide : "+item.Key);}
        var actual=Directory.GetFiles(stage,"*",SearchOption.AllDirectories).Select(p=>p.Substring(stage.TrimEnd(Path.DirectorySeparatorChar).Length+1).Replace(Path.DirectorySeparatorChar,'/')).Where(p=>p!="update-manifest.json").ToArray();
        if(actual.Length!=manifest.files.Count||actual.Any(p=>!manifest.files.ContainsKey(p)))throw new InvalidDataException("Le ZIP contient des fichiers non déclarés.");
        return manifest;
    }
    internal static string Stage(string root,string zip,ReleaseInfo release,CancellationToken cancel)
    {
        if(new FileInfo(zip).Length!=release.size||Hash(zip)!=release.digest)throw new InvalidDataException("L’empreinte du ZIP ne correspond pas à GitHub. Version installée conservée.");
        string name=".apm-stage-"+Guid.NewGuid().ToString("N"),stage=ManagedDirectory(root,name,".apm-stage-");Directory.CreateDirectory(stage);
        try{
            using(var archive=ZipFile.OpenRead(zip)){
                if(archive.Entries.Count>60)throw new InvalidDataException("Trop de fichiers dans le ZIP.");
                long expanded=0;var names=new HashSet<string>(StringComparer.OrdinalIgnoreCase);
                foreach(var entry in archive.Entries){
                    cancel.ThrowIfCancellationRequested();string relative=entry.FullName.Replace('\\','/');
                    if(relative!="update-manifest.json"&&!Allowed(relative))throw new InvalidDataException("Chemin interdit dans le ZIP : "+relative);
                    if(!names.Add(relative))throw new InvalidDataException("Fichier dupliqué dans le ZIP.");
                    expanded=checked(expanded+entry.Length);if(expanded>MaxExpanded)throw new InvalidDataException("Contenu décompressé trop volumineux.");
                    string path=relative=="update-manifest.json"?Path.Combine(stage,relative):Destination(stage,relative);Directory.CreateDirectory(Path.GetDirectoryName(path));
                    using(var input=entry.Open())using(var output=File.Create(path)){var buffer=new byte[64*1024];int count;long written=0;while((count=input.Read(buffer,0,buffer.Length))>0){cancel.ThrowIfCancellationRequested();written+=count;if(written>entry.Length||written>MaxExpanded)throw new InvalidDataException("Taille décompressée invalide.");output.Write(buffer,0,count);}if(written!=entry.Length)throw new InvalidDataException("Fichier incomplet.");}
                }
            }
            Manifest(stage,release.version);return stage;
        }catch{DeleteManaged(root,name,".apm-stage-");throw;}
    }
    private static string JournalPath(string root){return Path.Combine(root,".apm-update-journal.json");}
    private static void SaveJournal(string root,Journal journal)
    {
        string path=JournalPath(root),temporary=path+".tmp";
        using(var stream=new FileStream(temporary,FileMode.Create,FileAccess.Write,FileShare.None))using(var writer=new StreamWriter(stream,new UTF8Encoding(false))){writer.Write(Json.Serialize(journal));writer.Flush();stream.Flush(true);}
        if(File.Exists(path))File.Replace(temporary,path,null);else File.Move(temporary,path);
    }
    private static void AtomicCopy(string source,string target)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(target));string temporary=target+".apm-new";
        try{File.Copy(source,temporary,true);if(File.Exists(target))File.Replace(temporary,target,null);else File.Move(temporary,target);}
        finally{if(File.Exists(temporary))File.Delete(temporary);}
    }
    internal static void Recover(string root)
    {
        string path=JournalPath(root);if(!File.Exists(path))return;
        var journal=Json.Deserialize<Journal>(File.ReadAllText(path));
        if(journal==null||journal.files==null||journal.files.Count>50||(journal.phase!="applying"&&journal.phase!="committed"&&journal.phase!="rolledback"))throw new InvalidDataException("Journal de mise à jour invalide.");
        string backup=ManagedDirectory(root,journal.backup,".apm-backup-");
        if(journal.phase=="applying"){
            // Ensure every old file is available before restoring anything.
            foreach(var file in journal.files)if(file.Value&&!File.Exists(Destination(backup,file.Key)))throw new InvalidDataException("Sauvegarde de mise à jour incomplète.");
            foreach(var file in journal.files){string target=Destination(root,file.Key);if(file.Value)AtomicCopy(Destination(backup,file.Key),target);else if(File.Exists(target))File.Delete(target);}
            journal.phase="rolledback";SaveJournal(root,journal);
        }
        DeleteManaged(root,journal.backup,".apm-backup-");File.Delete(path);
    }
    internal static void Apply(string root,string stage,string version,Action<int> afterFile)
    {
        string name=Path.GetFileName(stage);if(ManagedDirectory(root,name,".apm-stage-")!=Path.GetFullPath(stage))throw new InvalidDataException("Dossier de préparation invalide.");
        Recover(root);var manifest=Manifest(stage,version);string backupName=".apm-backup-"+Guid.NewGuid().ToString("N"),backup=ManagedDirectory(root,backupName,".apm-backup-");Directory.CreateDirectory(backup);
        var journal=new Journal(){phase="applying",backup=backupName,files=new Dictionary<string,bool>()};
        try{
            foreach(string file in manifest.files.Keys){string target=Destination(root,file);bool exists=File.Exists(target);journal.files.Add(file,exists);if(exists)AtomicCopy(target,Destination(backup,file));}
            SaveJournal(root,journal);int index=0;
            foreach(string file in manifest.files.Keys){AtomicCopy(Destination(stage,file),Destination(root,file));if(afterFile!=null)afterFile(++index);}
            foreach(var file in manifest.files)if(Hash(Destination(root,file.Key))!=file.Value)throw new InvalidDataException("Vérification après remplacement échouée.");
            journal.phase="committed";SaveJournal(root,journal);
        }catch{if(File.Exists(JournalPath(root)))Recover(root);else DeleteManaged(root,backupName,".apm-backup-");throw;}
        Recover(root);DeleteManaged(root,name,".apm-stage-");
    }
    internal static bool MeterRunning(){return Process.GetProcessesByName("AionPartyMeter").Length>0;}
    internal static Mutex Lock(string root)
    {
        using(var sha=SHA256.Create())return new Mutex(false,"Local\\AionPartyMeter.Update."+BitConverter.ToString(sha.ComputeHash(Encoding.UTF8.GetBytes(Path.GetFullPath(root).ToLowerInvariant()))).Replace("-",""));
    }
}

internal static class GitHubTransport
{
    internal static void Fetch(string url,Stream output,long limit,CancellationToken cancel,Action<long> progress)
    {
        for(int redirects=0;redirects<6;redirects++){
            var uri=new Uri(url);if(uri.Scheme!="https"||(uri.Host!="api.github.com"&&uri.Host!="github.com"&&uri.Host!="release-assets.githubusercontent.com"&&uri.Host!="objects.githubusercontent.com"))throw new InvalidDataException("Destination réseau de mise à jour interdite.");
            var request=(HttpWebRequest)WebRequest.Create(uri);request.UserAgent="AionPartyMeter-Updater";request.Accept="application/vnd.github+json";request.Headers["X-GitHub-Api-Version"]="2026-03-10";request.Timeout=5000;request.ReadWriteTimeout=15000;request.AllowAutoRedirect=false;
            using(cancel.Register(request.Abort)){
                cancel.ThrowIfCancellationRequested();
                using(var response=(HttpWebResponse)request.GetResponse()){
                    if((int)response.StatusCode>=300&&(int)response.StatusCode<400){url=new Uri(uri,response.Headers["Location"]).AbsoluteUri;continue;}
                    if(response.ContentLength>limit)throw new InvalidDataException("Téléchargement trop volumineux.");
                    using(var input=response.GetResponseStream()){var buffer=new byte[64*1024];long total=0;int count;while((count=input.Read(buffer,0,buffer.Length))>0){cancel.ThrowIfCancellationRequested();total+=count;if(total>limit)throw new InvalidDataException("Téléchargement trop volumineux.");output.Write(buffer,0,count);if(progress!=null)progress(total);}}return;
                }
            }
        }
        throw new InvalidDataException("Trop de redirections GitHub.");
    }
    internal static ReleaseInfo Check(UpdateConfig config,Version installed,CancellationToken cancel)
    {
        if(!config.enabled||String.IsNullOrWhiteSpace(config.repository))return null;
        ServicePointManager.SecurityProtocol=SecurityProtocolType.Tls12;
        using(var output=new MemoryStream()){
            Fetch("https://api.github.com/repos/"+UpdateCore.Repository(config.repository)+"/releases/latest",output,2*1024*1024,cancel,null);
            return UpdateCore.SelectRelease(Encoding.UTF8.GetString(output.ToArray()),config.repository,installed);
        }
    }
}
