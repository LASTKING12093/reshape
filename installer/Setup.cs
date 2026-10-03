using System;
using System.IO;
using System.Linq;
using System.Collections.Generic;
using System.Threading;
using System.Net;
using System.Diagnostics;
using System.Reflection;
using System.Security.Cryptography;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Markup;
using System.Windows.Media;
using System.Windows.Media.Animation;
using System.Windows.Media.Imaging;
using System.Windows.Interop;
using Microsoft.Win32;

[assembly: AssemblyTitle("Reshape Setup")]
[assembly: AssemblyProduct("Reshape")]
[assembly: AssemblyVersion("2.0.4.0")]
[assembly: AssemblyFileVersion("2.0.4.0")]

class Setup {
    static readonly Assembly Assets = Assembly.GetExecutingAssembly();
    const string UninstallKey = @"Software\Microsoft\Windows\CurrentVersion\Uninstall\Forma";
    Window window; bool portuguese, busy, installed, failed, preview, upgrading;
    string destination;
    CancellationTokenSource cancellation; bool committing;
    Dictionary<string,EnginePackage> packages; Dictionary<string,string> engineFiles;
    static T Resource<T>(string name) { using(var reader=new StreamReader(Assets.GetManifestResourceStream(name))) return EngineSetup.Json.Deserialize<T>(reader.ReadToEnd()); }
    [DllImport("dwmapi.dll")] static extern int DwmSetWindowAttribute(IntPtr hwnd, int attribute, ref int value, int size);
    [DllImport("shell32.dll", CharSet=CharSet.Unicode)] static extern int SetCurrentProcessExplicitAppUserModelID(string id);
    [STAThread] static int Main(string[] args) {
        SetCurrentProcessExplicitAppUserModelID("com.reshape.setup");
        ServicePointManager.SecurityProtocol=SecurityProtocolType.Tls12;
        var setup = new Setup();
        setup.packages=Resource<Dictionary<string,EnginePackage>>("engines.lock");setup.engineFiles=Resource<Dictionary<string,string>>("engine.files");
        setup.destination = ExistingLocation();
        string path = args.FirstOrDefault(a => a.StartsWith("/D=", StringComparison.OrdinalIgnoreCase));
        if (path != null) setup.destination = path.Substring(3);
        if (args.Any(a => a.Equals("/S", StringComparison.OrdinalIgnoreCase))) {
            try { setup.cancellation=new CancellationTokenSource(); setup.Install(setup.destination, null); return 0; }
            catch (Exception e) { File.WriteAllText(Path.Combine(Path.GetTempPath(), "Reshape-setup-error.txt"), e.ToString()); return 1; }
        }
        setup.preview = args.Contains("--preview") || !Assets.GetManifestResourceNames().Contains("payload");
        var app = new Application();
        setup.Build();
        app.Run(setup.window);
        return 0;
    }
    static string ExistingLocation() {
        using (var key = Registry.CurrentUser.OpenSubKey(UninstallKey)) {
            var saved = key == null ? null : key.GetValue("InstallLocation") as string;
            if (!String.IsNullOrWhiteSpace(saved)) return saved.Trim('"');
        }
        return Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Programs", "Reshape");
    }
    T Get<T>(string name) where T : FrameworkElement { return (T)window.FindName(name); }
    string L(string en, string pt) { return portuguese ? pt : en; }
    void Text(string name, string en, string pt) { Get<TextBlock>(name).Text = L(en,pt); }
    void Label(string name, string en, string pt) { Get<Button>(name).Content = L(en,pt); }
    static BitmapImage Bitmap(string name) {
        using (var stream = Assets.GetManifestResourceStream(name)) {
            var image = new BitmapImage(); image.BeginInit(); image.CacheOption = BitmapCacheOption.OnLoad; image.StreamSource = stream; image.EndInit(); image.Freeze(); return image;
        }
    }
    static bool InButton(DependencyObject node) {
        while(node != null) { if(node is Button) return true; node = VisualTreeHelper.GetParent(node); } return false;
    }
    void Build() {
        using (var stream = Assets.GetManifestResourceStream("layout")) window = (Window)XamlReader.Load(stream);
        window.Icon = Bitmap("icon");
        Get<Image>("SmallLogo").Source = Bitmap("mark"); Get<Image>("HeroLogo").Source = Bitmap("mark");
        upgrading = File.Exists(Path.Combine(destination,"reshape.exe"));
        Get<TextBox>("LocationInput").Text = destination;
        Get<TextBlock>("LocationText").Text = destination;
        Get<Grid>("TitleArea").MouseLeftButtonDown += (s,e) => { if (!InButton(e.OriginalSource as DependencyObject)) { try { window.DragMove(); } catch (InvalidOperationException) {} } };
        Get<Button>("Close").Click += (s,e) => window.Close();
        Get<Button>("Minimize").Click += (s,e) => window.WindowState = WindowState.Minimized;
        Get<Button>("Language").Click += (s,e) => { portuguese = !portuguese; Translate(); };
        Get<Button>("Change").Click += (s,e) => { var input=Get<TextBox>("LocationInput"); input.Visibility=input.IsVisible ? Visibility.Collapsed : Visibility.Visible; if(input.IsVisible) { input.Focus(); input.SelectAll(); } };
        Get<TextBox>("LocationInput").TextChanged += (s,e) => { destination=Get<TextBox>("LocationInput").Text.Trim(); Get<TextBlock>("LocationText").Text=destination; };
        Get<Button>("Primary").Click += Start;
        Get<Button>("CancelWork").Click += (s,e) => { if(!committing && cancellation!=null) { cancellation.Cancel(); Get<Button>("CancelWork").IsEnabled=false; Text("Status","Cancelling safely…","Cancelando com segurança…"); } };
        window.Closing += (s,e) => { if(busy) { e.Cancel=true; Text("Detail", "Please let installation finish. You can minimize this window.", "Aguarde a instalação terminar. Você pode minimizar esta janela."); } };
        window.SourceInitialized += (s,e) => { int dark=1, corner=2; var hwnd=new WindowInteropHelper(window).Handle; DwmSetWindowAttribute(hwnd,20,ref dark,4); DwmSetWindowAttribute(hwnd,33,ref corner,4); };
        window.Loaded += (s,e) => { var body=Get<Grid>("Body"); body.BeginAnimation(UIElement.OpacityProperty,new DoubleAnimation(0,1,TimeSpan.FromMilliseconds(260))); };
        Translate();
    }
    void Translate() {
        window.Title=L("Reshape Setup", "Instalação do Reshape");
        Label("Language", "English  /  PT", "Português  /  EN");
        Get<Button>("Close").ToolTip=L("Close", "Fechar"); Get<Button>("Minimize").ToolTip=L("Minimize", "Minimizar");
        Text("LocationLabel", "INSTALL LOCATION", "LOCAL DE INSTALAÇÃO"); Label("Change", "Change", "Alterar");
        Text("Eyebrow", "LOCAL BY DESIGN", "LOCAL. PRIVADO. SEU.");
        Text("Headline", "A little space.\nA lot of possibility.", "Mais espaço.\nMais possibilidades.");
        Text("Description", "Convert and compress your files.\nEverything stays on your computer.", "Converta e comprima seus arquivos.\nTudo acontece no seu computador.");
        Text("Footer", preview ? "Design preview · No installation" : "Windows 64-bit · Internet needed for setup", preview ? "Prévia visual · Não instala arquivos" : "Windows 64 bits · Internet na instalação");
        Text("DownloadSize", "Download: up to " + (packages.Values.Sum(e=>e.bytes)/1000000) + " MB · Free space for setup: 6 GB", "Download: até " + (packages.Values.Sum(e=>e.bytes)/1000000) + " MB · Espaço livre para instalar: 6 GB");
        if(installed) {
            Text("Headline", "Ready to reshape.", "Tudo pronto.");
            Text("Description", "Your workspace is ready.\nDrop in a file and make it yours.", "Seu espaço está pronto.\nAdicione um arquivo para começar.");
            Text("Status", "Installation complete", "Instalação concluída");
            Text("Detail", "Your settings and history are preserved.", "Suas configurações e seu histórico foram preservados.");
            Label("Primary", "Open Reshape  →", "Abrir Reshape  →");
        } else Label("Primary", failed ? "Try again  →" : upgrading ? "Update Reshape  →" : "Install Reshape  →", failed ? "Tentar novamente  →" : upgrading ? "Atualizar Reshape  →" : "Instalar Reshape  →");
    }
    async void Start(object sender, RoutedEventArgs args) {
        if(installed) { try { Process.Start(new ProcessStartInfo(Path.Combine(destination,"reshape.exe")) { UseShellExecute=true }); window.Close(); } catch(Exception e) { MessageBox.Show(window,e.Message,"Reshape"); } return; }
        if(preview || busy) return;
        busy=true; failed=false; committing=false; cancellation=new CancellationTokenSource(); Get<Button>("CancelWork").Visibility=Visibility.Visible;Get<Button>("CancelWork").IsEnabled=true; Get<Button>("Primary").IsEnabled=false; Get<Button>("Language").IsEnabled=false;
        Get<Border>("LocationPanel").Visibility=Visibility.Collapsed; Get<StackPanel>("ProgressPanel").Visibility=Visibility.Visible;
        Text("Headline", "Making room\nfor what's next.", "Preparando seu\nnovo espaço.");
        Text("Description", "Downloading processing engines from their suppliers.\nYour files are processed locally.", "Baixando os motores diretamente dos fornecedores.\nSeus arquivos são processados localmente.");
        Text("Detail", "Your original files stay untouched.", "Seus arquivos originais permanecem intactos.");
        try {
            destination=Path.GetFullPath(destination);
            await Task.Run(() => Install(destination, progress => window.Dispatcher.Invoke(new Action(() => {
                double percent=progress.Total>0?progress.Done*100.0/progress.Total:-1;
                Get<ProgressBar>("Progress").IsIndeterminate=percent<0;
                if(percent>=0)Get<ProgressBar>("Progress").Value=percent;
                string phase=progress.Stage;
                if(portuguese){switch(phase){case "Preparing":phase="Preparando";break;case "Downloading components":phase="Baixando componentes";break;case "Verifying":phase="Verificando";break;case "Installing":phase="Instalando";break;case "Finishing":phase="Finalizando";break;}}
                Get<TextBlock>("Status").Text=phase+" · "+progress.Component;
                Get<TextBlock>("Detail").Text=progress.Total>0?String.Format(System.Globalization.CultureInfo.InvariantCulture,"{0:0}% · {1:0.0} MB / {2:0.0} MB · {3:0.0} MB/s",percent,progress.Done/1000000.0,progress.Total/1000000.0,progress.Speed/1000000.0):progress.Done>0?String.Format("{0:0.0} MB downloaded",progress.Done/1000000.0):L("Working. This stage does not report a percentage.","Processando. Esta etapa não informa porcentagem.");
                Get<Button>("CancelWork").IsEnabled=!committing;
            }))));
            installed=true; Get<ProgressBar>("Progress").IsIndeterminate=false; Get<ProgressBar>("Progress").Value=100;
        } catch(Exception e) {
            failed=true; Get<ProgressBar>("Progress").IsIndeterminate=false;
            Get<TextBlock>("Detail").Text=e is OperationCanceledException ? L("Cancelled. Temporary files were removed. You can try again.","Cancelado. Os temporários foram removidos. Você pode tentar novamente.") : e is WebException ? L("Reshape needs an internet connection to download its processing engines during setup. Check your connection and retry.","O Reshape precisa de internet para baixar os motores durante a instalação. Verifique a conexão e tente novamente.") : e.Message;
            Text("Status", "Installation could not finish", "Não foi possível concluir a instalação");
            Get<Border>("LocationPanel").Visibility=Visibility.Visible;
        } finally {
            busy=false; Get<Button>("CancelWork").Visibility=Visibility.Collapsed; Get<Button>("Primary").IsEnabled=true; Get<Button>("Language").IsEnabled=true; Translate();
        }
    }
    void Install(string path, Action<SetupProgress> progress) {
        using(var gate=new Mutex(false,@"Local\ReshapeSetup.Install")) {
            bool acquired=false;
            try {try{acquired=gate.WaitOne(0);}catch(AbandonedMutexException){acquired=true;}
                if(!acquired)throw new InvalidOperationException(L("Another Reshape installation is already running.","Outra instalação do Reshape já está em andamento."));
                InstallLocked(path,progress);
            } finally {if(acquired)gate.ReleaseMutex();}
        }
    }
    void InstallLocked(string path, Action<SetupProgress> progress) {
        if(String.IsNullOrWhiteSpace(path) || !Path.IsPathRooted(path) || Path.GetFullPath(path).TrimEnd('\\') == Path.GetPathRoot(path).TrimEnd('\\'))
            throw new InvalidOperationException(L("Choose an application folder, not a drive root.","Escolha uma pasta para o aplicativo, não a raiz do disco."));
        if(path.IndexOf('"')>=0) throw new InvalidOperationException(L("Invalid installation path.","Caminho de instalação inválido."));
        foreach(var p in Process.GetProcessesByName("reshape")) {
            using(p) { try { if(String.Equals(p.MainModule.FileName,Path.Combine(path,"reshape.exe"),StringComparison.OrdinalIgnoreCase)) throw new InvalidOperationException(L("Close Reshape, then try again. Active conversions must finish first.","Feche o Reshape e tente novamente. Aguarde as conversões em andamento terminarem.")); } catch(System.ComponentModel.Win32Exception) {} }
        }
        var token=cancellation.Token;
        Action<SetupProgress> report=p=>{if(progress!=null)progress(p);};
        string parent=Path.GetDirectoryName(Path.GetFullPath(path)); Directory.CreateDirectory(parent);
        if(new DriveInfo(Path.GetPathRoot(Path.GetFullPath(path))).AvailableFreeSpace<6000000000L)throw new IOException(L("Setup needs 6 GB of free disk space for downloads, extraction and updates.","A instalação precisa de 6 GB livres para downloads, extração e atualização."));
        string staging=Path.Combine(parent,".reshape-setup-"+Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(staging);
        bool coreStarted=false;
        string coreBackup=Path.Combine(staging,"previous-core");Directory.CreateDirectory(coreBackup);
        string[] coreNames={"reshape.exe","forma.exe","uninstall.exe"};
        foreach(string name in coreNames)if(File.Exists(Path.Combine(path,name)))File.Copy(Path.Combine(path,name),Path.Combine(coreBackup,name));
        try {
            EngineSetup.Prepare(packages,engineFiles,Path.Combine(path,"binaries"),staging,report,token);
            token.ThrowIfCancellationRequested();
            string payload=Path.Combine(staging,"core.exe");
            using(var input=Assets.GetManifestResourceStream("payload")) {
                if(input==null) throw new InvalidOperationException("Installer payload missing.");
                using(var output=new FileStream(payload,FileMode.CreateNew,FileAccess.Write,FileShare.None))
                using(var hash=SHA256.Create()) {
                    var buffer=new byte[1024*1024]; int count; long copied=0; int last=-1;
                    while((count=input.Read(buffer,0,buffer.Length))>0) { output.Write(buffer,0,count);hash.TransformBlock(buffer,0,count,null,0);copied+=count;int percent=(int)(copied*100/input.Length);if(percent!=last){last=percent;report(new SetupProgress("Preparing","Reshape",copied,input.Length));} }
                    report(new SetupProgress("Verifying","Reshape"));
                    hash.TransformFinalBlock(new byte[0],0,0);
                    string expected; using(var reader=new StreamReader(Assets.GetManifestResourceStream("payload.sha256"))) expected=reader.ReadToEnd().Trim();
                    if(!String.Equals(BitConverter.ToString(hash.Hash).Replace("-",""),expected,StringComparison.OrdinalIgnoreCase)) throw new InvalidOperationException(L("Installer integrity check failed. Obtain a fresh copy.","Falha na integridade do instalador. Obtenha uma nova cópia."));
                }
            }
            token.ThrowIfCancellationRequested(); committing=true; report(new SetupProgress("Installing","Reshape"));
            string runtime=Path.Combine(staging,"downloads","webview2.exe");
            if(File.Exists(runtime))EngineSetup.Run(runtime,"/silent /install",CancellationToken.None);
            if(!EngineSetup.HasWebView())throw new InvalidOperationException("WebView2 could not be verified. Retry setup.");
            using(var key=Registry.CurrentUser.CreateSubKey(@"Software\forma\Forma")) key.SetValue("Installer Language",portuguese ? 1046 : 1033,RegistryValueKind.DWord);
            coreStarted=true;
            using(var engine=Process.Start(new ProcessStartInfo(payload,"/S /UPDATE /D="+path) { UseShellExecute=false,CreateNoWindow=true,WindowStyle=ProcessWindowStyle.Hidden })) {
                engine.WaitForExit();if(engine.ExitCode!=0)throw new InvalidOperationException(L("Installation failed (code ","A instalação falhou (código ")+engine.ExitCode+L("). Close Reshape and try again.","). Feche o Reshape e tente novamente."));
            }
            report(new SetupProgress("Finishing","Reshape"));
            string engineTarget=Path.Combine(path,"binaries"), backup=Path.Combine(staging,"previous-engines");
            string app=Path.Combine(path,"reshape.exe");
            if(!File.Exists(app) || !FileVersionInfo.GetVersionInfo(app).ProductVersion.StartsWith("2.0.4")) throw new InvalidOperationException(L("Installed application could not be verified.","Não foi possível verificar o aplicativo instalado."));
            EngineSetup.CommitEngines(Path.Combine(staging,"binaries"),engineTarget,backup,()=>{
                foreach(var component in packages.Values.Where(e=>e.required))if(!EngineSetup.VerifyEngine(engineTarget,component.id,engineFiles,CancellationToken.None))throw new IOException("Installed component verification failed: "+component.name);
            });
        } catch {
            if(coreStarted)foreach(string name in coreNames){string previous=Path.Combine(coreBackup,name),current=Path.Combine(path,name);if(File.Exists(previous))File.Copy(previous,current,true);else if(File.Exists(current))File.Delete(current);}
            throw;
        } finally {
            // Only the unique directory created above; never remove the destination.
            try { Directory.Delete(staging,true); } catch(IOException) {} catch(UnauthorizedAccessException) {}
        }
    }
}
