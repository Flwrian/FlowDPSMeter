// GPL-3.0-only. Isolated files, no network, capture, installation or game process.
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.IO.Compression;
using System.Threading;

internal static class UpdaterTests
{
    private static int passed;private static string directory;
    private static void Check(bool value,string label){if(!value)throw new Exception(label);passed++;Console.WriteLine("PASS "+label);}
    private static void Reject(Action action,string label){try{action();}catch{Check(true,label);return;}throw new Exception("Accepted: "+label);}
    private static string Fixture(Dictionary<string,string> files,string version)
    {
        string root=Path.Combine(directory,"case-"+Guid.NewGuid().ToString("N"));Directory.CreateDirectory(root);
        string stage=Path.Combine(root,".apm-stage-"+Guid.NewGuid().ToString("N"));Directory.CreateDirectory(stage);
        var hashes=new Dictionary<string,string>();
        foreach(var file in files){string path=UpdateCore.Destination(stage,file.Key);Directory.CreateDirectory(Path.GetDirectoryName(path));File.WriteAllText(path,file.Value);hashes[file.Key]=UpdateCore.Hash(path);}
        File.WriteAllText(Path.Combine(stage,"update-manifest.json"),UpdateCore.Json.Serialize(new UpdateManifest(){schemaVersion=1,version=version,files=hashes}));return stage;
    }
    private static Dictionary<string,string> Files(){return new Dictionary<string,string>{{"AionPartyMeter.exe","new meter"},{"DEMARRER.exe","new launcher"},{"SHA256.json","{}"},{"licences/GPL-3.0.txt","GPL"}};}
    private static string Release(string tag,bool prerelease,string repo,string digest,string url)
    {
        return UpdateCore.Json.Serialize(new{tag_name=tag,draft=false,prerelease=prerelease,assets=new[]{new{name="AionPartyMeter-"+tag.TrimStart('v')+"-Update-Windows-x64.zip",browser_download_url=url,digest=digest,size=1024}}});
    }
    private static int Main(string[] args)
    {
        try{
            directory=Path.Combine(Path.GetFullPath(args[0]),"updater-test-"+Guid.NewGuid().ToString("N"));Directory.CreateDirectory(directory);
            string repo="owner/AionPartyMeter",tag="v0.1.9",url="https://github.com/"+repo+"/releases/download/"+tag+"/AionPartyMeter-0.1.9-Update-Windows-x64.zip",digest="sha256:"+new string('a',64);
            Check(UpdateCore.Repository("https://github.com/owner/AionPartyMeter.git/")==repo,"GitHub URL normalized");
            Reject(()=>UpdateCore.Repository("https://example.com/repo"),"foreign configuration rejected");Reject(()=>UpdateCore.Repository("owner/../secret"),"repository traversal rejected");
            Check(GitHubTransport.Check(new UpdateConfig(){enabled=false,repository=repo},new Version(0,1,8),CancellationToken.None)==null,"disabled updates make no HTTP request");
            Check(GitHubTransport.Check(new UpdateConfig(){repository=""},new Version(0,1,8),CancellationToken.None)==null,"unconfigured updates make no HTTP request");
            Check(UpdateCore.SelectRelease(Release(tag,false,repo,digest,url),repo,new Version(0,1,8)).version=="0.1.9","stable newer release selected");
            Check(UpdateCore.SelectRelease(Release(tag,false,repo,digest,url),repo,new Version(0,1,9))==null,"same version skipped");
            Check(UpdateCore.SelectRelease(Release(tag,false,repo,digest,url),repo,new Version(0,2,0))==null,"downgrade skipped");
            Check(UpdateCore.SelectRelease(Release(tag,true,repo,digest,url),repo,new Version(0,1,8))==null,"prerelease skipped");
            Reject(()=>UpdateCore.VersionOf("v0.1.9-beta"),"unstable tag rejected");
            Reject(()=>UpdateCore.SelectRelease(Release(tag,false,repo,null,url),repo,new Version(0,1,8)),"missing GitHub digest rejected");
            Reject(()=>UpdateCore.SelectRelease(Release(tag,false,repo,digest,url.Replace("owner/","other/")),repo,new Version(0,1,8)),"other repository asset rejected");
            Reject(()=>UpdateCore.Destination(directory,"../settings.json"),"zip traversal rejected");Reject(()=>UpdateCore.Destination(directory,"settings.json"),"local settings never replaced");Reject(()=>UpdateCore.Destination(directory,"captures/test.apm.jsonl"),"local captures never replaced");
            string stage=Fixture(Files(),"0.1.9"),root=Path.GetDirectoryName(stage);
            File.WriteAllText(Path.Combine(root,"AionPartyMeter.exe"),"old meter");File.WriteAllText(Path.Combine(root,"DEMARRER.exe"),"old launcher");File.WriteAllText(Path.Combine(root,"settings.json"),"settings");File.WriteAllText(Path.Combine(root,"history.json"),"history");File.WriteAllText(Path.Combine(root,"update.json"),"config");
            UpdateCore.Apply(root,stage,"0.1.9",null);
            Check(File.ReadAllText(Path.Combine(root,"AionPartyMeter.exe"))=="new meter"&&File.ReadAllText(Path.Combine(root,"DEMARRER.exe"))=="new launcher","meter and launcher updated together");
            Check(File.ReadAllText(Path.Combine(root,"settings.json"))=="settings"&&File.ReadAllText(Path.Combine(root,"history.json"))=="history"&&File.ReadAllText(Path.Combine(root,"update.json"))=="config","settings, history and repository preserved");
            Check(!File.Exists(Path.Combine(root,".apm-update-journal.json"))&&!Directory.Exists(stage),"successful transaction cleaned up");
            stage=Fixture(Files(),"0.1.9");root=Path.GetDirectoryName(stage);File.WriteAllText(Path.Combine(root,"AionPartyMeter.exe"),"old meter");File.WriteAllText(Path.Combine(root,"DEMARRER.exe"),"old launcher");
            string rollbackStage=stage;Reject(()=>UpdateCore.Apply(Path.GetDirectoryName(rollbackStage),rollbackStage,"0.1.9",index=>{if(index==3)throw new IOException("Injected interruption");}),"injected failure rolls back");
            Check(File.ReadAllText(Path.Combine(root,"AionPartyMeter.exe"))=="old meter"&&File.ReadAllText(Path.Combine(root,"DEMARRER.exe"))=="old launcher"&&!File.Exists(Path.Combine(root,"SHA256.json")),"old version restored, newly added files removed");
            stage=Fixture(Files(),"0.1.9");File.AppendAllText(Path.Combine(stage,"AionPartyMeter.exe"),"tampered");string tampered=stage;Reject(()=>UpdateCore.Manifest(tampered,"0.1.9"),"tampered staged file rejected");
            stage=Fixture(Files(),"0.1.9");string different=stage;Reject(()=>UpdateCore.Manifest(different,"0.1.8"),"manifest version mismatch rejected");
            stage=Fixture(Files(),"0.1.9");root=Path.GetDirectoryName(stage);string zip=Path.Combine(root,"update.zip");ZipFile.CreateFromDirectory(stage,zip);var release=new ReleaseInfo(){size=new FileInfo(zip).Length,digest=UpdateCore.Hash(zip),version="0.1.9"};
            string extracted=UpdateCore.Stage(root,zip,release,CancellationToken.None);Check(UpdateCore.Manifest(extracted,"0.1.9").files.Count==4,"ZIP fully staged and hashed before replacement");
            using(var cancelled=new CancellationTokenSource()){cancelled.Cancel();Reject(()=>UpdateCore.Stage(root,zip,release,cancelled.Token),"cancelled staging never replaces installation");}
            release.digest=new string('b',64);Reject(()=>UpdateCore.Stage(root,zip,release,CancellationToken.None),"GitHub ZIP digest mismatch rejected");
            string hostile=Path.Combine(root,"hostile.zip");using(var z=ZipFile.Open(hostile,ZipArchiveMode.Create)){using(var writer=new StreamWriter(z.CreateEntry("../outside.txt").Open()))writer.Write("bad");}
            release.size=new FileInfo(hostile).Length;release.digest=UpdateCore.Hash(hostile);Reject(()=>UpdateCore.Stage(root,hostile,release,CancellationToken.None),"malicious archive cannot escape stage");
            string duplicate=Path.Combine(root,"duplicate.zip");using(var z=ZipFile.Open(duplicate,ZipArchiveMode.Create)){z.CreateEntry("DEMARRER.exe");z.CreateEntry("DEMARRER.exe");}
            release.size=new FileInfo(duplicate).Length;release.digest=UpdateCore.Hash(duplicate);Reject(()=>UpdateCore.Stage(root,duplicate,release,CancellationToken.None),"duplicate ZIP entry rejected");
            stage=Fixture(Files(),"0.1.9");root=Path.GetDirectoryName(stage);string backupName=".apm-backup-"+Guid.NewGuid().ToString("N"),backup=Path.Combine(root,backupName);Directory.CreateDirectory(backup);File.WriteAllText(Path.Combine(root,"AionPartyMeter.exe"),"partial new");File.WriteAllText(Path.Combine(backup,"AionPartyMeter.exe"),"pre-crash old");
            File.WriteAllText(Path.Combine(root,".apm-update-journal.json"),UpdateCore.Json.Serialize(new Journal(){phase="applying",backup=backupName,files=new Dictionary<string,bool>{{"AionPartyMeter.exe",true}}}));UpdateCore.Recover(root);Check(File.ReadAllText(Path.Combine(root,"AionPartyMeter.exe"))=="pre-crash old","next startup recovers interrupted transaction");
            foreach(string phase in new[]{"committed","rolledback"}){
                stage=Fixture(Files(),"0.1.9");root=Path.GetDirectoryName(stage);backupName=".apm-backup-"+Guid.NewGuid().ToString("N");Directory.CreateDirectory(Path.Combine(root,backupName));File.WriteAllText(Path.Combine(root,"AionPartyMeter.exe"),"keep verified version");
                File.WriteAllText(Path.Combine(root,".apm-update-journal.json"),UpdateCore.Json.Serialize(new Journal(){phase=phase,backup=backupName,files=new Dictionary<string,bool>{{"AionPartyMeter.exe",true}}}));UpdateCore.Recover(root);
                Check(File.ReadAllText(Path.Combine(root,"AionPartyMeter.exe"))=="keep verified version"&&!Directory.Exists(Path.Combine(root,backupName))&&!File.Exists(Path.Combine(root,".apm-update-journal.json")),"cleanup after "+phase+" interruption is idempotent");
            }
            stage=Fixture(Files(),"0.1.9");root=Path.GetDirectoryName(stage);File.WriteAllText(Path.Combine(root,"AionPartyMeter.exe"),"locked old");
            using(var locked=File.Open(Path.Combine(root,"AionPartyMeter.exe"),FileMode.Open,FileAccess.Read,FileShare.None)){string lockedStage=stage;Reject(()=>UpdateCore.Apply(Path.GetDirectoryName(lockedStage),lockedStage,"0.1.9",null),"locked meter leaves installation unchanged");}
            Check(File.ReadAllText(Path.Combine(root,"AionPartyMeter.exe"))=="locked old"&&!File.Exists(Path.Combine(root,"DEMARRER.exe")),"failed preflight writes no program files");
            // Exercise the real replacement helper in another process without launching any UI or meter.
            stage=Fixture(Files(),"0.1.9");root=Path.GetDirectoryName(stage);string helper=Path.Combine(Path.GetFullPath(args[0]),"DEMARRER.exe");
            using(var p=Process.Start(new ProcessStartInfo(helper,"--apply-update "+UpdateStartup.Quote(root)+" "+UpdateStartup.Quote(stage)+" 0 0.1.9 --no-launch"){UseShellExecute=false,CreateNoWindow=true})){Check(p.WaitForExit(10000)&&p.ExitCode==0&&File.ReadAllText(Path.Combine(root,"DEMARRER.exe"))=="new launcher","replacement helper process succeeds without starting capture");}
            Console.WriteLine(passed+" updater checks passed. Isolated fixture directory: "+directory);return 0;
        }catch(Exception e){Console.Error.WriteLine(e);return 1;}
    }
}
