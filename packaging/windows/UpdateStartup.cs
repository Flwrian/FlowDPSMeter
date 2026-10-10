// GPL-3.0-only. Runs before capture; never terminates a running meter or game.
using System;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Forms;

internal sealed class LaunchResult { public bool handoff; public string message; }
internal static class UpdateStartup
{
    internal static string Quote(string value){return "\""+value.Replace("\"","\\\"").TrimEnd('\\')+"\"";}
    internal static void Result(string message)
    {
        try{string directory=Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),"AionPartyMeter");Directory.CreateDirectory(directory);File.WriteAllText(Path.Combine(directory,"update-last-result.txt"),DateTime.UtcNow.ToString("u")+"\n"+message);}catch(IOException){}catch(UnauthorizedAccessException){}
    }
    internal static LaunchResult Run(bool skip,CancellationToken cancel,Action<string> progress)
    {
        string root=Launcher.Root;
        using(var mutex=UpdateCore.Lock(root)){
            bool locked;try{locked=mutex.WaitOne(0);}catch(AbandonedMutexException){locked=true;}
            if(!locked)return new LaunchResult(){handoff=true,message="Une mise à jour est déjà en cours. Réessaie dans un instant."};
            try{
                if(UpdateCore.MeterRunning())return new LaunchResult(){handoff=true,message="Le meter est déjà ouvert. Ferme-le avant de le relancer pour installer une mise à jour."};
                UpdateCore.Recover(root);
                var config=UpdateCore.Config(root);
                if(skip||!config.enabled||String.IsNullOrWhiteSpace(config.repository))return new LaunchResult(){message=skip?"Mise à jour ignorée pour ce lancement.":!config.enabled?"Mises à jour automatiques désactivées.":"Dépôt GitHub non configuré."};
                progress("Recherche d’une nouvelle version sur GitHub…");
                ReleaseInfo release=GitHubTransport.Check(config,UpdateCore.Installed(root),cancel);
                if(release==null)return new LaunchResult(){message="La version installée est à jour."};
                string zip=Path.Combine(Path.GetTempPath(),"AionPartyMeter-"+Guid.NewGuid().ToString("N")+".zip"),stage=null;
                try{
                    progress("Téléchargement de la version "+release.version+"…");
                    using(var output=File.Create(zip))GitHubTransport.Fetch(release.url,output,UpdateCore.MaxZip,cancel,bytes=>progress("Version "+release.version+" · "+(100*bytes/release.size)+" %"));
                    progress("Vérification du téléchargement…");stage=UpdateCore.Stage(root,zip,release,cancel);
                    if(UpdateCore.Installed(stage)!=UpdateCore.VersionOf(release.version))throw new InvalidDataException("La version de l’exécutable ne correspond pas à la release.");
                    cancel.ThrowIfCancellationRequested();
                    if(UpdateCore.MeterRunning())throw new IOException("Le meter a été ouvert pendant le téléchargement. La mise à jour attendra le prochain lancement.");
                    string helperDirectory=Path.Combine(Path.GetTempPath(),"AionPartyMeter-helper-"+Guid.NewGuid().ToString("N"));Directory.CreateDirectory(helperDirectory);
                    string helper=Path.Combine(helperDirectory,"DEMARRER.exe");File.Copy(Path.Combine(root,"DEMARRER.exe"),helper);
                    progress("Installation de la mise à jour, puis ouverture du meter…");
                    var process=Process.Start(new ProcessStartInfo(helper,"--apply-update "+Quote(root)+" "+Quote(stage)+" "+Process.GetCurrentProcess().Id+" "+release.version){UseShellExecute=false,CreateNoWindow=true,WorkingDirectory=root});
                    if(process==null)throw new IOException("L’assistant de remplacement n’a pas démarré.");process.Dispose();stage=null;
                    return new LaunchResult(){handoff=true,message="Mise à jour "+release.version+" transmise à l’assistant de remplacement."};
                }finally{
                    if(File.Exists(zip))File.Delete(zip);
                    if(stage!=null)UpdateCore.DeleteManaged(root,Path.GetFileName(stage),".apm-stage-");
                }
            }finally{mutex.ReleaseMutex();}
        }
    }
    internal static int ApplyHelper(string[] args)
    {
        string root=Path.GetFullPath(args[1]),stage=Path.GetFullPath(args[2]);bool test=args.Length==6&&args[5]=="--no-launch";
        try{
            int parent=int.Parse(args[3]);
            if(parent<=0&&!test)throw new InvalidDataException("Processus de lancement invalide.");
            if(parent>0)try{using(var p=Process.GetProcessById(parent)){if(!p.WaitForExit(15000))throw new IOException("Le lanceur précédent est encore actif.");}}catch(ArgumentException){}
            using(var mutex=UpdateCore.Lock(root)){
                bool locked;try{locked=mutex.WaitOne(15000);}catch(AbandonedMutexException){locked=true;}
                if(!locked)throw new IOException("Un autre lanceur effectue une mise à jour.");
                // Test fixtures live in an isolated directory; they never launch a meter.
                try{if(!test&&UpdateCore.MeterRunning())throw new IOException("Le meter est ouvert : aucun fichier n’a été remplacé.");UpdateCore.Apply(root,stage,args[4],null);}
                finally{mutex.ReleaseMutex();}
            }
            if(!test){Result("Version "+args[4]+" installée. Réglages et historique conservés.");Process.Start(new ProcessStartInfo(Path.Combine(root,"DEMARRER.exe"),"--skip-update"){UseShellExecute=true,WorkingDirectory=root});}
            return 0;
        }catch(Exception e){
            if(!test){
                Result("Mise à jour non appliquée : "+e.Message);
                // Recover() must have completed before opening an installation after a failure.
                if(!File.Exists(Path.Combine(root,".apm-update-journal.json"))&&!UpdateCore.MeterRunning()&&File.Exists(Path.Combine(root,"DEMARRER.exe"))){
                    try{Process.Start(new ProcessStartInfo(Path.Combine(root,"DEMARRER.exe"),"--skip-update"){UseShellExecute=true,WorkingDirectory=root});return 1;}catch(Exception launch){Result("Mise à jour non appliquée : "+e.Message+" Ouverture impossible : "+launch.Message);}
                }
                MessageBox.Show("La mise à jour n’a pas pu être installée.\n"+e.Message+"\nRelance DEMARRER.exe pour restaurer et ouvrir la version précédente.","AionPartyMeter — mise à jour",MessageBoxButtons.OK,MessageBoxIcon.Warning);
            }
            return 1;
        }
    }
}

internal sealed class UpdateWindow : Form
{
    private readonly Label status=new Label();private readonly Button skip=new Button();private readonly CancellationTokenSource cancel=new CancellationTokenSource();
    private bool completed;private readonly bool bypass;
    internal UpdateWindow(bool skipUpdate)
    {
        bypass=skipUpdate;Text="AionPartyMeter — lancement";ClientSize=new Size(460,155);FormBorderStyle=FormBorderStyle.FixedDialog;MaximizeBox=false;MinimizeBox=false;StartPosition=FormStartPosition.CenterScreen;
        Font=new Font("Segoe UI",10);BackColor=Color.FromArgb(19,27,37);ForeColor=Color.White;
        status.Text="Préparation du meter…";status.SetBounds(22,22,418,55);Controls.Add(status);
        skip.Text="Ouvrir sans mise à jour";skip.SetBounds(22,96,240,34);skip.Click+=delegate{skip.Enabled=false;status.Text="Ouverture de la version installée…";cancel.Cancel();};Controls.Add(skip);
        FormClosing+=delegate(object sender,FormClosingEventArgs e){if(!completed){cancel.Cancel();e.Cancel=true;}};
        Shown+=async delegate{
            LaunchResult result;
            cancel.CancelAfter(TimeSpan.FromMinutes(3));
            try{result=await Task.Run(()=>UpdateStartup.Run(bypass,cancel.Token,text=>{if(!IsDisposed)BeginInvoke((Action)(()=>status.Text=text));}));}
            catch(Exception e){result=new LaunchResult(){message=cancel.IsCancellationRequested?"Mise à jour ignorée : ouverture de la version installée.":"Vérification indisponible : "+e.Message+" La version installée reste utilisable."};}
            UpdateStartup.Result(result.message);status.Text=result.message;
            completed=true;
            if(result.handoff){Close();return;}
            // A pending rollback must succeed before launching a possibly mixed pack.
            if(File.Exists(Path.Combine(Launcher.Root,".apm-update-journal.json"))){MessageBox.Show(this,"Une restauration de mise à jour reste nécessaire. Relance le lanceur ou extrais à nouveau le pack. Tes données locales sont conservées.","AionPartyMeter",MessageBoxButtons.OK,MessageBoxIcon.Warning);Close();return;}
            try{if(Launcher.WebViewAvailable()&&Launcher.NpcapAvailable())Launcher.OpenMeter(false);else{Hide();using(var setup=new SetupWindow())setup.ShowDialog();}}
            catch(Exception e){MessageBox.Show(this,e.Message,"AionPartyMeter",MessageBoxButtons.OK,MessageBoxIcon.Error);}
            Close();
        };
    }
    protected override void Dispose(bool disposing){if(disposing)cancel.Dispose();base.Dispose(disposing);}
}

internal sealed class UpdateSettingsWindow : Form
{
    private readonly TextBox repository=new TextBox();private readonly CheckBox enabled=new CheckBox();
    internal UpdateSettingsWindow()
    {
        Text="AionPartyMeter — mises à jour GitHub";ClientSize=new Size(540,255);StartPosition=FormStartPosition.CenterParent;Font=new Font("Segoe UI",10);FormBorderStyle=FormBorderStyle.FixedDialog;MaximizeBox=false;
        var config=UpdateCore.Config(Launcher.Root);
        Controls.Add(new Label(){Text="Dépôt GitHub public (compte/dépôt ou URL)",Location=new Point(20,20),AutoSize=true});
        repository.SetBounds(20,50,495,28);repository.Text=config.repository;Controls.Add(repository);
        enabled.Text="Vérifier et installer les versions stables au lancement";enabled.Checked=config.enabled;enabled.SetBounds(20,93,500,30);Controls.Add(enabled);
        Controls.Add(new Label(){Text="Aucune donnée de combat n’est envoyée. Sans Internet, la version installée s’ouvre. La mise à jour attend si le meter est déjà ouvert.",Location=new Point(20,132),Size=new Size(495,52)});
        var save=new Button(){Text="Enregistrer",Location=new Point(20,202),Size=new Size(160,32)};save.Click+=delegate{try{string value=repository.Text.Trim();if(value.Length>0)value=UpdateCore.Repository(value);File.WriteAllText(Path.Combine(Launcher.Root,"update.json"),UpdateCore.Json.Serialize(new UpdateConfig(){enabled=enabled.Checked,repository=value}),new System.Text.UTF8Encoding(false));Close();}catch(Exception e){MessageBox.Show(this,e.Message,"Configuration",MessageBoxButtons.OK,MessageBoxIcon.Warning);}};Controls.Add(save);
    }
}
