using System;
using System.IO;
using System.Collections.Generic;
using System.Net;
using System.Threading;

// Developer/CI entry point. Not included in the shipped bootstrapper.
class PrepareEngines {
 static int Main(string[] args) {
  try {
   if(args.Length!=4)throw new ArgumentException("Expected manifest, file hashes, existing directory, staging directory.");
   ServicePointManager.SecurityProtocol=SecurityProtocolType.Tls12;
   var packages=EngineSetup.Json.Deserialize<Dictionary<string,EnginePackage>>(File.ReadAllText(args[0]));
   var files=EngineSetup.Json.Deserialize<Dictionary<string,string>>(File.ReadAllText(args[1]));
   long last=-1;string stage="";
   EngineSetup.Prepare(packages,files,Path.GetFullPath(args[2]),Path.GetFullPath(args[3]),p=>{
    long percent=p.Total>0?p.Done*100/p.Total:-1;
    if(stage!=p.Stage+ p.Component || percent!=last){Console.WriteLine(EngineSetup.Json.Serialize(p));stage=p.Stage+p.Component;last=percent;}
   },CancellationToken.None);
   return 0;
  }catch(Exception e){Console.Error.WriteLine(e);return 1;}
 }
}
