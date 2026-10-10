// AionPartyMeter launcher, GPL-3.0-only. Uses the .NET Framework supplied with Windows 10/11.
using System;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Net;
using System.Runtime.InteropServices;
using System.Security.Cryptography.X509Certificates;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using System.Windows.Forms;
using Microsoft.Win32;

internal static class Launcher
{
    internal static readonly string Root = AppDomain.CurrentDomain.BaseDirectory;
    internal static string Meter { get { return Path.Combine(Root, "AionPartyMeter.exe"); } }
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    private static extern IntPtr LoadLibraryEx(string name, IntPtr reserved, uint flags);
    [DllImport("kernel32.dll")] private static extern bool FreeLibrary(IntPtr handle);

    internal static bool NpcapAvailable()
    {
        string system=Environment.GetFolderPath(Environment.SpecialFolder.System);
        foreach(string path in new[]{Path.Combine(system,"Npcap","wpcap.dll"),Path.Combine(system,"wpcap.dll")}) {
            if(!File.Exists(path))continue;
            IntPtr handle=LoadLibraryEx(path,IntPtr.Zero,0x100|0x800);
            if(handle!=IntPtr.Zero){FreeLibrary(handle);return true;}
        }
        return false;
    }
    internal static bool WebViewAvailable()
    {
        const string key="SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
        foreach(RegistryHive hive in new[]{RegistryHive.LocalMachine,RegistryHive.CurrentUser})
            foreach(RegistryView view in new[]{RegistryView.Registry32,RegistryView.Registry64})
                try { using(RegistryKey root=RegistryKey.OpenBaseKey(hive,view)) using(RegistryKey client=root.OpenSubKey(key)) {
                    Version version; if(client!=null&&Version.TryParse(client.GetValue("pv","").ToString(),out version)&&version.Major>0)return true;
                }} catch(System.Security.SecurityException){} catch(UnauthorizedAccessException){}
        return false;
    }
    internal static void OpenMeter(bool admin)
    {
        if(!File.Exists(Meter))throw new FileNotFoundException("Extrais tout le ZIP avant de lancer le pack.",Meter);
        ProcessStartInfo start=new ProcessStartInfo(Meter){WorkingDirectory=Root,UseShellExecute=true};
        if(admin)start.Verb="runas";
        Process.Start(start);
    }
    internal static Uri NpcapDownload(string page)
    {
        Match match=Regex.Match(page,@"href\s*=\s*[""'](?:https://npcap\.com/|/)?(dist/npcap-[0-9]+(?:\.[0-9]+)+\.exe)[""']",RegexOptions.IgnoreCase);
        if(!match.Success)throw new InvalidOperationException("Le lien officiel Npcap a changé. Ouvre https://npcap.com/#download depuis le guide.");
        return new Uri(new Uri("https://npcap.com/"),match.Groups[1].Value);
    }
    [STAThread] private static int Main(string[] args)
    {
        if(args.Length>=5&&args[0]=="--apply-update")return UpdateStartup.ApplyHelper(args);
        if(args.Length==2&&args[0]=="--update-check"){
            try{var config=UpdateCore.Config(Root);var installed=UpdateCore.Installed(Root);var release=GitHubTransport.Check(config,installed,System.Threading.CancellationToken.None);File.WriteAllText(args[1],UpdateCore.Json.Serialize(new{repository=config.repository,enabled=config.enabled,installed=installed.ToString(),available=release!=null,version=release==null?null:release.version}));return 0;}
            catch(Exception e){File.WriteAllText(args[1],UpdateCore.Json.Serialize(new{error=e.Message}));return 1;}
        }
        if(args.Length==2&&args[0]=="--check") {
            File.WriteAllText(args[1],"{\"windowsX64\":"+Environment.Is64BitOperatingSystem.ToString().ToLowerInvariant()+
                ",\"webview2\":"+WebViewAvailable().ToString().ToLowerInvariant()+",\"npcap\":"+NpcapAvailable().ToString().ToLowerInvariant()+
                ",\"meter\":"+File.Exists(Meter).ToString().ToLowerInvariant()+"}",new UTF8Encoding(false));return 0;
        }
        if(args.Length==2&&(args[0]=="--verify-microsoft"||args[0]=="--verify-npcap")) {try{Signature.Verify(args[1],args[0]=="--verify-microsoft");return 0;}catch{return 1;}}
        if(args.Length==3&&args[0]=="--npcap-link") {File.WriteAllText(args[2],NpcapDownload(File.ReadAllText(args[1])).AbsoluteUri);return 0;}
        Application.EnableVisualStyles();Application.SetCompatibleTextRenderingDefault(false);
        if(!Environment.Is64BitOperatingSystem){MessageBox.Show("Ce pack nécessite Windows 10/11 x64.","AionPartyMeter");return 1;}
        try {
            if(args.Length==0||args.Length==1&&args[0]=="--skip-update")Application.Run(new UpdateWindow(args.Length==1));
            else if(args.Length==1&&args[0]=="--updates")Application.Run(new UpdateSettingsWindow());
            else Application.Run(new SetupWindow());
            return 0;
        }catch(Exception e){MessageBox.Show(e.Message,"AionPartyMeter",MessageBoxButtons.OK,MessageBoxIcon.Error);return 1;}
    }
}

internal sealed class SetupWindow : Form
{
    private readonly Label web=new Label(),npcap=new Label(),message=new Label();
    private readonly Button webInstall=new Button(),npcapInstall=new Button(),launch=new Button(),admin=new Button(),refresh=new Button();
    private bool busy;
    public SetupWindow()
    {
        Text="AionPartyMeter — configuration";ClientSize=new Size(610,465);MinimumSize=new Size(626,504);
        StartPosition=FormStartPosition.CenterScreen;Font=new Font("Segoe UI",10);BackColor=Color.FromArgb(19,27,37);ForeColor=Color.White;
        Label title=new Label(){Text="Tout le groupe, au même endroit.",Location=new Point(24,22),AutoSize=true,Font=new Font("Segoe UI",17,FontStyle.Bold)};
        Label subtitle=new Label(){Text="Les dépendances sont vérifiées avant l’ouverture du meter.",Location=new Point(26,61),AutoSize=true,ForeColor=Color.LightGray};
        Controls.Add(title);Controls.Add(subtitle);
        web.SetBounds(26,111,550,28);npcap.SetBounds(26,147,550,28);Controls.Add(web);Controls.Add(npcap);
        webInstall.Text="Installer WebView2 (fourni)";webInstall.SetBounds(26,189,262,38);
        npcapInstall.Text="Télécharger et installer Npcap";npcapInstall.SetBounds(302,189,280,38);
        webInstall.Click+=async delegate{await RunWork(InstallWebView);};npcapInstall.Click+=async delegate{await RunWork(InstallNpcap);};
        Controls.Add(webInstall);Controls.Add(npcapInstall);
        message.SetBounds(26,242,556,65);message.Text="Npcap nécessite Internet et une installation par Windows. Son installateur n’est pas redistribué dans le pack. Accepte sa licence et l’invite administrateur ; les options par défaut conviennent.";
        message.ForeColor=Color.LightGray;Controls.Add(message);
        launch.Text="Ouvrir le meter";launch.SetBounds(26,321,168,38);launch.Click+=delegate{Launch(false);};
        admin.Text="Ouvrir en administrateur";admin.SetBounds(204,321,215,38);admin.Click+=delegate{Launch(true);};
        refresh.Text="Vérifier à nouveau";refresh.SetBounds(429,321,153,38);refresh.Click+=delegate{RefreshStatus();};
        Controls.Add(launch);Controls.Add(admin);Controls.Add(refresh);
        Button help=new Button(){Text="Mode d’emploi",Location=new Point(26,376),Size=new Size(168,30)};
        help.Click+=delegate{Process.Start(new ProcessStartInfo(Path.Combine(Launcher.Root,"LIRE-MOI.html")){UseShellExecute=true});};Controls.Add(help);
        Button updates=new Button(){Text="Mises à jour GitHub",Location=new Point(204,376),Size=new Size(215,30)};
        updates.Click+=delegate{try{using(var dialog=new UpdateSettingsWindow())dialog.ShowDialog(this);}catch(Exception e){MessageBox.Show(this,e.Message,"Mises à jour");}};Controls.Add(updates);
        foreach(Control control in Controls)if(control is Button){control.BackColor=Color.FromArgb(39,58,70);control.ForeColor=Color.White;((Button)control).FlatStyle=FlatStyle.Flat;}
        FormClosing+=delegate(object sender,FormClosingEventArgs e){if(busy)e.Cancel=true;};
        RefreshStatus();
    }
    private void RefreshStatus()
    {
        bool w=Launcher.WebViewAvailable(),n=Launcher.NpcapAvailable();
        web.Text="WebView2 : "+(w?"présent":"à installer — installateur hors ligne fourni");
        npcap.Text="Npcap : "+(n?"bibliothèque disponible":"à installer pour la capture réseau");
        web.ForeColor=w?Color.LightGreen:Color.Wheat;npcap.ForeColor=n?Color.LightGreen:Color.Wheat;
        webInstall.Enabled=!busy&&!w;npcapInstall.Enabled=!busy&&!n;launch.Enabled=!busy&&w;admin.Enabled=!busy&&w;refresh.Enabled=!busy;
    }
    private void Launch(bool elevated)
    {
        try{Launcher.OpenMeter(elevated);Close();}catch(Exception e){MessageBox.Show(this,e.Message,"Lancement",MessageBoxButtons.OK,MessageBoxIcon.Error);}
    }
    private async Task RunWork(Func<Task> operation)
    {
        busy=true;RefreshStatus();UseWaitCursor=true;
        try{await operation();message.Text="Installation terminée. Les dépendances ont été vérifiées à nouveau. Si Windows demande un redémarrage, redémarre puis relance DEMARRER.exe.";}
        catch(Exception e){message.Text=e.Message+" Tu peux suivre le Mode d’emploi pour une installation manuelle.";}
        finally{busy=false;UseWaitCursor=false;RefreshStatus();}
    }
    private async Task InstallWebView()
    {
        string path=Path.Combine(Launcher.Root,"dependances","MicrosoftEdgeWebView2RuntimeInstallerX64.exe");
        message.Text="Vérification de la signature Microsoft puis installation de WebView2…";
        await Task.Run(delegate{Signature.Verify(path,true);});
        int result=await Task.Run(delegate{using(Process p=Process.Start(new ProcessStartInfo(path,"/silent /install"){UseShellExecute=false,CreateNoWindow=true})){p.WaitForExit();return p.ExitCode;}});
        if(!Launcher.WebViewAvailable())throw new InvalidOperationException("WebView2 n’a pas été détecté après installation (code "+result+").");
    }
    private async Task InstallNpcap()
    {
        message.Text="Téléchargement depuis npcap.com… L’installateur officiel va ensuite demander ton accord.";
        ServicePointManager.SecurityProtocol=SecurityProtocolType.Tls12;
        string directory=Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),"AionPartyMeter","installateurs");
        Directory.CreateDirectory(directory);
        using(WebClient client=new WebClient()) {
            string page=await client.DownloadStringTaskAsync(new Uri("https://npcap.com/"));
            Uri url=Launcher.NpcapDownload(page);
            string path=Path.Combine(directory,Path.GetFileName(url.AbsolutePath));
            await client.DownloadFileTaskAsync(url,path);
            await Task.Run(delegate{Signature.Verify(path,false);});
            await Task.Run(delegate{using(Process p=Process.Start(new ProcessStartInfo(path){UseShellExecute=true,Verb="runas"})){p.WaitForExit();}});
        }
        if(!Launcher.NpcapAvailable())throw new InvalidOperationException("Npcap n’est pas encore disponible. L’installation a pu être annulée ou demander un redémarrage.");
    }
}

internal static class Signature
{
    [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)] private struct FileInfo {public uint size;public string path;public IntPtr file,subject;}
    [StructLayout(LayoutKind.Sequential)] private struct TrustData {
        public uint size;public IntPtr callback,sip;public uint ui,revoke,choice;public IntPtr file;
        public uint state;public IntPtr stateData,url;public uint flags,context;public IntPtr settings;
    }
    [DllImport("wintrust.dll",ExactSpelling=true,PreserveSig=true)] private static extern int WinVerifyTrust(IntPtr window,ref Guid action,ref TrustData data);
    internal static void Verify(string path,bool microsoft)
    {
        if(!File.Exists(path))throw new FileNotFoundException("Installateur absent. Extrais tout le pack.",path);
        FileInfo file=new FileInfo(){size=(uint)Marshal.SizeOf(typeof(FileInfo)),path=path};
        IntPtr memory=Marshal.AllocHGlobal(Marshal.SizeOf(typeof(FileInfo)));Marshal.StructureToPtr(file,memory,false);
        TrustData trust=new TrustData(){size=(uint)Marshal.SizeOf(typeof(TrustData)),ui=2,choice=1,file=memory,state=1,flags=0x1000};
        Guid action=new Guid("00AAC56B-CD44-11D0-8CC2-00C04FC295EE");
        try{
            int status=WinVerifyTrust(new IntPtr(-1),ref action,ref trust);
            if(status!=0)throw new InvalidOperationException("Signature de l’installateur non validée par Windows (0x"+status.ToString("X8")+"). Aucun installateur n’a été lancé.");
            using(X509Certificate2 certificate=new X509Certificate2(X509Certificate.CreateFromSignedFile(path))){
                string publisher=certificate.GetNameInfo(X509NameType.SimpleName,false);
                bool expected=microsoft?publisher=="Microsoft Corporation":publisher=="Nmap Software LLC"||publisher=="Insecure.Com LLC";
                if(!expected)throw new InvalidOperationException("Éditeur inattendu : "+publisher+". Aucun installateur n’a été lancé.");
            }
        }finally{trust.state=2;WinVerifyTrust(new IntPtr(-1),ref action,ref trust);Marshal.DestroyStructure(memory,typeof(FileInfo));Marshal.FreeHGlobal(memory);}
    }
}
