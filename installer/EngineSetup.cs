using System;
using System.IO;
using System.Linq;
using System.Net;
using System.Collections.Generic;
using System.Diagnostics;
using System.Security.Cryptography;
using System.Threading;
using System.Web.Script.Serialization;
using System.IO.Compression;
using Microsoft.Win32;

public class EnginePackage {
 public string id, name, version, license, project, url, source, kind, folder, executable, architecture, distribution;
 public string sha256; public long bytes; public bool required;
}
public class SetupProgress {
 public string Stage, Component; public long Done, Total; public double Speed;
 public SetupProgress(string stage,string component,long done=0,long total=0,double speed=0){Stage=stage;Component=component;Done=done;Total=total;Speed=speed;}
}
public static class EngineSetup {
 public const string OfficeVersion="26.2.6";
 public const string OfficeHash="f9877032fd908beb9c0ddf06df4af5c2e85f419c42e14876c4cce5aae5fb2660";
 public static void ValidateRelease(Dictionary<string,EnginePackage> packages) {
  var office=packages.Values.Single(e=>e.id=="libreoffice");
  if(office.version!=OfficeVersion || office.sha256!=OfficeHash || office.bytes!=373252096)throw new InvalidOperationException("This Reshape release requires the authenticated LibreOffice 26.2.6 package. Its version, size and SHA-256 must match the release lock.");
 }
 public static readonly JavaScriptSerializer Json = new JavaScriptSerializer { MaxJsonLength=16000000 };
 public static string Hash(string path) { using(var s=File.OpenRead(path))using(var h=SHA256.Create())return BitConverter.ToString(h.ComputeHash(s)).Replace("-","").ToLowerInvariant(); }
 public static bool ValidFile(string path,EnginePackage e) {return File.Exists(path) && (e.bytes<=0 || new FileInfo(path).Length==e.bytes) && Hash(path)==e.sha256;}
 public static WebResponse Request(Uri url) {
  for(int redirects=0;redirects<8;redirects++) {
   if(url.Scheme!="https")throw new InvalidOperationException("Only HTTPS component sources are allowed.");
   var req=(HttpWebRequest)WebRequest.Create(url);req.AllowAutoRedirect=false;req.Timeout=30000;req.ReadWriteTimeout=30000;req.UserAgent="Reshape-Setup/2.0.4";
   var response=(HttpWebResponse)req.GetResponse();
   int status=(int)response.StatusCode;
   if(status>=300 && status<400){string next=response.Headers["Location"];response.Dispose();if(String.IsNullOrEmpty(next))throw new IOException("Invalid supplier redirect.");url=new Uri(url,next);continue;}
   if(status<200 || status>=300){response.Dispose();throw new IOException("The supplier returned an unsuccessful response.");}
   return response;
  }
  throw new IOException("Too many supplier redirects.");
 }
 // The optional transport is used by the separate test executable, never by setup command-line arguments.
 public static void Download(EnginePackage e,string path,Action<SetupProgress> report,CancellationToken cancel,Func<Uri,WebResponse> transport=null,Func<string,Stream> outputFactory=null) {
  if(ValidFile(path,e))return;
  string part=path+".part"; if(File.Exists(part))File.Delete(part);
  try {
   cancel.ThrowIfCancellationRequested();
   using(var response=(transport??Request)(new Uri(e.url)))
   using(var registration=cancel.Register(()=>response.Close()))
   using(var input=response.GetResponseStream())
   using(var output=outputFactory==null?new FileStream(part,FileMode.CreateNew,FileAccess.Write,FileShare.None):outputFactory(part)) {
    long total=e.bytes>0?e.bytes:response.ContentLength;
    if(e.bytes>0 && response.ContentLength>0 && e.bytes!=response.ContentLength)throw new InvalidDataException("The component size differs from the pinned manifest.");
    byte[] buffer=new byte[128*1024];long done=0;int count;var watch=Stopwatch.StartNew();long last=0;
    while((count=input.Read(buffer,0,buffer.Length))>0){cancel.ThrowIfCancellationRequested();if(watch.Elapsed.TotalMinutes>30)throw new IOException("The component download timed out. Check your connection and retry.");output.Write(buffer,0,count);done+=count;if(total>0 && done>total)throw new InvalidDataException("Component is larger than expected.");if(watch.ElapsedMilliseconds-last>=100){last=watch.ElapsedMilliseconds;if(report!=null)report(new SetupProgress("Downloading components",e.name,done,total,done/Math.Max(.001,watch.Elapsed.TotalSeconds)));}}
    cancel.ThrowIfCancellationRequested();if(total>0 && done!=total)throw new InvalidDataException("The component download was interrupted.");
    if(report!=null)report(new SetupProgress("Downloading components",e.name,done,total,done/Math.Max(.001,watch.Elapsed.TotalSeconds)));
   }
   if(report!=null)report(new SetupProgress("Verifying",e.name));
   cancel.ThrowIfCancellationRequested();if(!ValidFile(part,e))throw new InvalidDataException("The component failed its SHA-256 integrity check. Retry to obtain a fresh copy.");
   if(File.Exists(path))File.Delete(path);File.Move(part,path);
  } catch { if(File.Exists(part))File.Delete(part);cancel.ThrowIfCancellationRequested();throw; }
 }
 public static void CopyTree(string source,string target,CancellationToken cancel) {
  Directory.CreateDirectory(target);
  foreach(var file in Directory.GetFiles(source)){cancel.ThrowIfCancellationRequested();if((File.GetAttributes(file)&FileAttributes.ReparsePoint)!=0)throw new IOException("Unexpected linked engine file.");File.Copy(file,Path.Combine(target,Path.GetFileName(file)),true);}
  foreach(var dir in Directory.GetDirectories(source)){cancel.ThrowIfCancellationRequested();if((File.GetAttributes(dir)&FileAttributes.ReparsePoint)!=0)throw new IOException("Unexpected linked engine directory.");CopyTree(dir,Path.Combine(target,Path.GetFileName(dir)),cancel);}
 }
 public static void Run(string exe,string args,CancellationToken cancel,int timeoutSeconds=600) {
  var info=new ProcessStartInfo(exe,args){UseShellExecute=false,CreateNoWindow=true,WindowStyle=ProcessWindowStyle.Hidden,RedirectStandardOutput=true,RedirectStandardError=true};
  using(var p=new Process{StartInfo=info}) {
   p.OutputDataReceived+=(s,e)=>{};p.ErrorDataReceived+=(s,e)=>{};
   p.Start();p.BeginOutputReadLine();p.BeginErrorReadLine();var clock=Stopwatch.StartNew();
   while(!p.WaitForExit(100)){if(cancel.IsCancellationRequested || clock.Elapsed.TotalSeconds>timeoutSeconds){try{p.Kill();p.WaitForExit();}catch{}cancel.ThrowIfCancellationRequested();throw new IOException("A component operation timed out. Retry setup.");}}
   if(p.ExitCode!=0 && p.ExitCode!=3010)throw new IOException(Path.GetFileName(exe)+" failed (code "+p.ExitCode+"). Retry setup.");
  }
 }
 public static void Unzip(string zip,string target,CancellationToken cancel) {
  Directory.CreateDirectory(target);string root=Path.GetFullPath(target)+Path.DirectorySeparatorChar;
  using(var archive=ZipFile.OpenRead(zip))foreach(var e in archive.Entries){cancel.ThrowIfCancellationRequested();string dest=Path.GetFullPath(Path.Combine(root,e.FullName));if(!dest.StartsWith(root,StringComparison.OrdinalIgnoreCase))throw new InvalidDataException("Unsafe archive path.");if(String.IsNullOrEmpty(e.Name)){Directory.CreateDirectory(dest);continue;}Directory.CreateDirectory(Path.GetDirectoryName(dest));using(var input=e.Open())using(var output=File.Create(dest)){byte[] b=new byte[131072];int n;while((n=input.Read(b,0,b.Length))>0){cancel.ThrowIfCancellationRequested();output.Write(b,0,n);}}}
 }
 public static bool VerifyEngine(string root,string id,Dictionary<string,string> expected,CancellationToken cancel) {
  var entries=expected.Where(p=>p.Key.StartsWith(id+"/",StringComparison.Ordinal)).ToArray();if(entries.Length==0)return false;
  foreach(var pair in entries){cancel.ThrowIfCancellationRequested();string path=Path.Combine(root,pair.Key.Replace('/',Path.DirectorySeparatorChar));if(!File.Exists(path)||Hash(path)!=pair.Value)return false;}
  return true;
 }
 public static bool HasWebView() {
  string[] clients={@"Software\Microsoft\EdgeUpdate\Clients",@"Software\WOW6432Node\Microsoft\EdgeUpdate\Clients"};
  foreach(var hive in new[]{Registry.CurrentUser,Registry.LocalMachine})foreach(string clientsPath in clients)using(var root=hive.OpenSubKey(clientsPath))if(root!=null)foreach(var name in root.GetSubKeyNames())using(var key=root.OpenSubKey(name)){string product=Convert.ToString(key.GetValue("name"));string version=Convert.ToString(key.GetValue("pv"));Version parsed;if(product.IndexOf("WebView2",StringComparison.OrdinalIgnoreCase)>=0 && Version.TryParse(version,out parsed)&&parsed.Major>=120)return true;}
  return false;
 }
 public static void CommitEngines(string prepared,string target,string backup,Action verify) {
  bool previous=Directory.Exists(target),moved=false;
  if(previous)Directory.Move(target,backup);
  try {Directory.Move(prepared,target);moved=true;verify();}
  catch {if(moved)Directory.Move(target,prepared);if(previous)Directory.Move(backup,target);throw;}
 }
 public static List<EnginePackage> Needed(Dictionary<string,EnginePackage> packages,string existing,Dictionary<string,string> files,CancellationToken cancel) {
  var needed=packages.Values.Where(e=>e.required && !VerifyEngine(existing,e.id,files,cancel)).ToList();
  if(needed.Any(e=>e.id=="7zip" || e.id=="imagemagick")){needed.Insert(0,packages.Values.Single(e=>e.id=="7zr"));if(!needed.Any(e=>e.id=="7zip") && !File.Exists(Path.Combine(existing,"7zip","7z.exe")))needed.Add(packages.Values.Single(e=>e.id=="7zip"));}
  if(needed.Any(e=>e.id=="libreoffice"))needed.Add(packages.Values.Single(e=>e.id=="lessmsi"));
  if(!HasWebView())needed.Add(packages.Values.Single(e=>e.id=="webview2"));return needed;
 }
 public static void Prepare(Dictionary<string,EnginePackage> packages,Dictionary<string,string> expected,string existing,string staging,Action<SetupProgress> report,CancellationToken cancel) {
  // The 25.8 package used by early local builds has known document-import vulnerabilities.
  // Fail closed even if an old lock file accidentally reaches a release build.
  ValidateRelease(packages);
  string downloads=Path.Combine(staging,"downloads"), prepared=Path.Combine(staging,"binaries");Directory.CreateDirectory(downloads);Directory.CreateDirectory(prepared);
  report(new SetupProgress("Preparing","Checking installed components"));
  var needed=Needed(packages,existing,expected,cancel);long allBytes=needed.All(e=>e.bytes>0)?needed.Sum(e=>e.bytes):-1,complete=0;
  foreach(var e in needed){string path=Path.Combine(downloads,e.id+Path.GetExtension(new Uri(e.url).AbsolutePath));Download(e,path,p=>{if(p.Stage=="Downloading components"){p.Done+=complete;p.Total=allBytes;}report(p);},cancel);complete+=e.bytes;}
  foreach(var e in packages.Values.Where(p=>p.required).OrderBy(p=>p.id=="7zip"?0:1)) {
   cancel.ThrowIfCancellationRequested();report(new SetupProgress("Installing",e.name));
   string dest=Path.Combine(prepared,e.id);
   if(!needed.Any(n=>n.id==e.id)){CopyTree(Path.Combine(existing,e.id),dest,cancel);continue;}
   string source=Path.Combine(downloads,e.id+Path.GetExtension(new Uri(e.url).AbsolutePath));
   string unpack=Path.Combine(staging,"unpack-"+e.id);
   if(e.kind=="zip") {
    Unzip(source,unpack,cancel);string extracted=Path.Combine(unpack,e.folder);
    if(e.id=="ffmpeg"){Directory.CreateDirectory(dest);CopyTree(Path.Combine(extracted,"bin"),dest,cancel);foreach(var f in Directory.GetFiles(extracted))File.Copy(f,Path.Combine(dest,Path.GetFileName(f)),true);}
    else Directory.Move(extracted,dest);
   } else if(e.id=="7zip") {
    Run(Path.Combine(downloads,"7zr.exe"),"x \""+source+"\" -o\""+dest+"\" -y",cancel);
   } else if(e.id=="imagemagick") {
    // 7-Zip is prepared first below; an already verified existing copy is also usable.
    string seven=Path.Combine(prepared,"7zip","7z.exe");if(!File.Exists(seven))seven=Path.Combine(existing,"7zip","7z.exe");
    Run(seven,"x \""+source+"\" -o\""+dest+"\" -y",cancel);
   } else if(e.id=="libreoffice") {
    string helper=Path.Combine(staging,"lessmsi");Unzip(Path.Combine(downloads,"lessmsi.zip"),helper,cancel);
    Directory.CreateDirectory(unpack);
    // lessmsi recognizes an output directory only with a trailing separator.
    // Double it before the closing quote for Windows command-line parsing.
    Run(Path.Combine(helper,"lessmsi.exe"),"x \""+source+"\" \""+unpack+"\\\\\"",cancel);
    Directory.Move(Path.Combine(unpack,"SourceDir","LibreOffice"),dest);
    foreach(var file in Directory.GetFiles(Path.Combine(unpack,"SourceDir","System64"),"*.dll"))File.Copy(file,Path.Combine(dest,"program",Path.GetFileName(file)),true);
   }
  }
  string omp=Path.Combine(prepared,"poppler","Library","bin","vcomp140.dll");File.Copy(omp,Path.Combine(prepared,"imagemagick","vcomp140.dll"),true);
  string policy=Path.Combine(prepared,"imagemagick","policy.xml"),xml=File.ReadAllText(policy);
  if(!xml.Contains("Reshape local-only policy"))File.WriteAllText(policy,xml.Replace("</policymap>","<!-- Reshape local-only policy --><policy domain=\"delegate\" rights=\"none\" pattern=\"*\"/><policy domain=\"filter\" rights=\"none\" pattern=\"*\"/><policy domain=\"coder\" rights=\"none\" pattern=\"{HTTP,HTTPS,FTP,URL,MSL,MVG,EPHEMERAL}\"/><policy domain=\"path\" rights=\"none\" pattern=\"@*\"/><policy domain=\"resource\" name=\"memory\" value=\"256MiB\"/><policy domain=\"resource\" name=\"map\" value=\"512MiB\"/><policy domain=\"resource\" name=\"disk\" value=\"4GiB\"/><policy domain=\"resource\" name=\"time\" value=\"300\"/></policymap>"));
  report(new SetupProgress("Verifying","Installed component files"));
  foreach(var e in packages.Values.Where(p=>p.required))if(!VerifyEngine(prepared,e.id,expected,cancel))throw new InvalidDataException("Extracted component integrity check failed: "+e.name);
  File.WriteAllText(Path.Combine(prepared,"manifest.json"),Json.Serialize(expected));File.WriteAllText(Path.Combine(prepared,".reshape-engines"),"com.reshape.desktop");
 }
}
