using System;
using System.IO;
using System.Net;
using System.Text;
using System.Threading;
using System.Collections.Generic;
using System.IO.Compression;

class EngineSetupTests {
 class FullDisk:MemoryStream {public override void Write(byte[] buffer,int offset,int count){throw new IOException("Simulated disk full");}}
 class Response:WebResponse {
  public Stream Data;public long Size;
  public override long ContentLength { get{return Size;} set{Size=value;} }
  public override Stream GetResponseStream(){return Data;}
  public override void Close(){Data.Dispose();}
 }
 static int passed;
 static void Assert(bool value,string name){if(!value)throw new Exception(name);passed++;Console.WriteLine("PASS "+name);}
 static void Fails(Action action,string name){bool failed=false;try{action();}catch(IOException){failed=true;}catch(InvalidDataException){failed=true;}catch(WebException){failed=true;}catch(OperationCanceledException){failed=true;}Assert(failed,name);}
 static int Main(){
  string root=Path.Combine(Path.GetTempPath(),"reshape-tests-"+Guid.NewGuid().ToString("N"));Directory.CreateDirectory(root);
  try {
   byte[] data=Encoding.UTF8.GetBytes("verified engine fixture");string original=Path.Combine(root,"original");File.WriteAllBytes(original,data);
   var e=new EnginePackage{id="test",name="Test",bytes=data.Length,sha256=EngineSetup.Hash(original),url="https://example.invalid/fixture"};
   string dest=Path.Combine(root,"engine");
   Func<Uri,WebResponse> good=u=>new Response{Data=new MemoryStream(data),Size=data.Length};
   var progress=new List<SetupProgress>();
   EngineSetup.Download(e,dest,p=>progress.Add(p),CancellationToken.None,good);
   Assert(EngineSetup.ValidFile(dest,e)&&!File.Exists(dest+".part"),"atomic verified download");
   Assert(progress.Exists(p=>p.Done==data.Length&&p.Total==data.Length),"progress reports real bytes");
   EngineSetup.Download(e,dest,null,CancellationToken.None,u=>{throw new Exception("cache was not reused");});Assert(true,"verified archive reuse");
   File.Delete(dest);File.WriteAllText(dest+".part","stale");
   EngineSetup.Download(e,dest,null,CancellationToken.None,good);Assert(EngineSetup.ValidFile(dest,e),"partial file replaced on retry");File.Delete(dest);
   Fails(()=>EngineSetup.Download(e,dest,null,CancellationToken.None,u=>new Response{Data=new MemoryStream(new byte[data.Length]),Size=data.Length}),"invalid SHA rejected");
   Assert(!File.Exists(dest)&&!File.Exists(dest+".part"),"invalid SHA cleanup");
   Fails(()=>EngineSetup.Download(e,dest,null,CancellationToken.None,u=>new Response{Data=new MemoryStream(new byte[2]),Size=data.Length}),"interrupted stream rejected");
   Assert(!File.Exists(dest+".part"),"interrupted stream cleanup");
   Fails(()=>EngineSetup.Download(e,dest,null,CancellationToken.None,u=>{throw new WebException("server unavailable");}),"server failure surfaces");
   var cancel=new CancellationTokenSource();
   Fails(()=>EngineSetup.Download(e,dest,p=>cancel.Cancel(),cancel.Token,good),"cancellation before commit");
   Assert(!File.Exists(dest)&&!File.Exists(dest+".part"),"cancelled download cleanup");
   Fails(()=>EngineSetup.Download(e,dest,null,CancellationToken.None,good,p=>new FullDisk()),"disk-full write failure surfaces");
   Assert(!File.Exists(dest)&&!File.Exists(dest+".part"),"disk-full cleanup");
   EngineSetup.Download(e,dest,null,CancellationToken.None,good);Assert(EngineSetup.ValidFile(dest,e),"retry after cancellation");File.Delete(dest);
   // Same filename and length, but bytes from an old release must never be reused.
   File.WriteAllBytes(dest,new byte[data.Length]);int fetched=0;
   EngineSetup.Download(e,dest,null,CancellationToken.None,u=>{fetched++;return good(u);});
   Assert(fetched==1&&EngineSetup.ValidFile(dest,e),"old LibreOffice cache replaced by locked artifact");File.Delete(dest);
   var office=new EnginePackage{id="libreoffice",version=EngineSetup.OfficeVersion,sha256=EngineSetup.OfficeHash,bytes=373252096};
   var release=new Dictionary<string,EnginePackage>{{"libreoffice.msi",office}};
   EngineSetup.ValidateRelease(release);Assert(true,"exact authenticated LibreOffice release accepted");
   foreach(string version in new[]{"25.8.4.2","26.2.5.2","26.8.0"}){office.version=version;bool rejected=false;try{EngineSetup.ValidateRelease(release);}catch(InvalidOperationException){rejected=true;}Assert(rejected,"unapproved release rejected: "+version);}
   office.version=EngineSetup.OfficeVersion;office.sha256=new string('0',64);bool wrongHash=false;try{EngineSetup.ValidateRelease(release);}catch(InvalidOperationException){wrongHash=true;}Assert(wrongHash,"correct version with wrong hash rejected");
   e.bytes=0;progress.Clear();EngineSetup.Download(e,dest,p=>progress.Add(p),CancellationToken.None,u=>new Response{Data=new MemoryStream(data),Size=-1});
   Assert(progress.Exists(p=>p.Done==data.Length&&p.Total<0),"unknown length remains indeterminate");
   Directory.CreateDirectory(Path.Combine(root,"test"));File.Copy(original,Path.Combine(root,"test","engine.dll"));
   var expected=new Dictionary<string,string>{{"test/engine.dll",e.sha256}};
   Assert(EngineSetup.VerifyEngine(root,"test",expected,CancellationToken.None),"installed engine verified");
   File.WriteAllText(Path.Combine(root,"test","engine.dll"),"corrupt");Assert(!EngineSetup.VerifyEngine(root,"test",expected,CancellationToken.None),"corrupt installed DLL rejected");
   File.Delete(Path.Combine(root,"test","engine.dll"));File.WriteAllText(Path.Combine(root,"test","old-engine.dll"),"old");Assert(!EngineSetup.VerifyEngine(root,"test",expected,CancellationToken.None),"stale extracted directory rejected");
   string brokenZip=Path.Combine(root,"broken.zip");File.WriteAllText(brokenZip,"truncated archive");Fails(()=>EngineSetup.Unzip(brokenZip,Path.Combine(root,"broken"),CancellationToken.None),"extraction failure surfaces");
   string zip=Path.Combine(root,"unsafe.zip");using(var a=ZipFile.Open(zip,ZipArchiveMode.Create)){using(var w=new StreamWriter(a.CreateEntry("../escape.txt").Open()))w.Write("unsafe");}
   Fails(()=>EngineSetup.Unzip(zip,Path.Combine(root,"unzip"),CancellationToken.None),"zip traversal rejected");Assert(!File.Exists(Path.Combine(root,"escape.txt")),"archive cannot escape destination");
   string prepared=Path.Combine(root,"prepared"),target=Path.Combine(root,"installed"),backup=Path.Combine(root,"backup");Directory.CreateDirectory(prepared);Directory.CreateDirectory(target);File.WriteAllText(Path.Combine(prepared,"version"),"26.2.6");File.WriteAllText(Path.Combine(target,"version"),"previous");
   Fails(()=>EngineSetup.CommitEngines(prepared,target,backup,()=>{throw new IOException("simulated commit validation failure");}),"commit validation failure surfaces");
   Assert(File.ReadAllText(Path.Combine(target,"version"))=="previous"&&Directory.Exists(prepared)&&!Directory.Exists(backup),"failed engine commit rolls back completely");
   EngineSetup.CommitEngines(prepared,target,backup,()=>{});Assert(File.ReadAllText(Path.Combine(target,"version"))=="26.2.6"&&!Directory.Exists(prepared),"successful commit activates only new directory");
   Console.WriteLine(passed+" installer checks passed.");return 0;
  }catch(Exception e){Console.Error.WriteLine(e);return 1;}finally{Directory.Delete(root,true);}
 }
}

