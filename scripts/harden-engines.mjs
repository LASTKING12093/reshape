import{readFileSync,writeFileSync}from'node:fs';
const path='src-tauri/binaries/imagemagick/policy.xml';let xml=readFileSync(path,'utf8');
if(!xml.includes('Forma local-only policy'))xml=xml.replace('</policymap>',`<!-- Forma local-only policy -->
<policy domain="delegate" rights="none" pattern="*"/>
<policy domain="filter" rights="none" pattern="*"/>
<policy domain="coder" rights="none" pattern="{HTTP,HTTPS,FTP,URL,MSL,MVG,EPHEMERAL}"/>
<policy domain="path" rights="none" pattern="@*"/>
<policy domain="resource" name="memory" value="256MiB"/>
<policy domain="resource" name="map" value="512MiB"/>
<policy domain="resource" name="disk" value="4GiB"/>
<policy domain="resource" name="time" value="300"/>
</policymap>`);
writeFileSync(path,xml);
