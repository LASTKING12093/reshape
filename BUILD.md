# Building Reshape on Windows

Requirements: Windows x64, Node.js 22+, Rust MSVC, Visual Studio C++ Build Tools and Windows SDK. Internet is needed to prepare dependencies.

```powershell
npm ci
powershell -ExecutionPolicy Bypass -File scripts/prepare-engines.ps1
node scripts/fixtures.mjs
npm test
npm run tauri -- dev
```

Engine preparation uses the same C# extraction and integrity code as setup. It downloads fixed supplier packages, authenticates them against `engines.lock.json`, and verifies the extracted files against `installer/engine-files.json`. No system LibreOffice installation or administrator MSI extraction is needed. Processing engines stay private under `binaries`; user-installed office software is not used.

```powershell
npm run lint
npm run build
powershell -ExecutionPolicy Bypass -File scripts/test-setup.ps1
npm run package
```

`npm run package` runs checks and builds the Tauri core and WPF online bootstrapper. The final candidate is `release/Reshape-Setup-2.0.4.exe`. The NSIS file under `src-tauri/target/release/bundle/nsis` is an internal core payload, not the public installer. The standalone app executable needs its managed `binaries` directory.

## Online installer

The Tauri resource list contains only legal notices and the Windows icon. No processing-engine executable, DLL or WebView2 runtime installer is embedded. The bootstrapper downloads engines directly from suppliers, checks fixed sizes and SHA-256, extracts into staging, verifies the extracted inventory and then installs them. WebView2 is downloaded from the pinned Microsoft artifact only when no suitable runtime exists.

Download progress uses actual received bytes and fixed artifact sizes. Unknown totals are indeterminate. Setup reports verification and extraction separately. It needs 6 GB of temporary free space; the installed engine footprint is smaller. English is the default and Portuguese is available.

Downloads and generated artifacts are ignored by Git. Do not upload processing-engine caches or old offline packages to the Reshape release. `docs/ONLINE_RELEASE_AUDIT.md` is the publication gate; creating a local test build does not clear that gate.

## Deliberate engine updates

Authenticate a replacement before updating its version, URL, size and hash. For LibreOffice 26.2.6, see `docs/LIBREOFFICE_AUTHENTICATION.md`. Regenerate the complete extracted inventory from a fresh authenticated tree with `node scripts/installer-file-inventory.mjs PATH`, and the binary report with `scripts/engine-inventory.ps1`. Never relabel old hashes. Update the explicit release policy and tests when intentionally changing a pinned release.

Cargo and npm lock files pin application dependencies. Release builds remap developer filesystem paths. Bit-for-bit reproducibility across toolchain and timestamp changes is not promised. No signing certificate is fabricated: the Reshape installer is unsigned unless the owner supplies a real certificate.

For legal-notice maintenance, generate Windows Cargo metadata into `.cache/cargo-metadata.json`, run `scripts/collect-licenses.mjs` and `scripts/fetch-license-notices.mjs`, and inspect coverage before packaging.

For the installed processing smoke test:

```powershell
Start-Process -FilePath 'INSTALL_LOCATION\reshape.exe' -ArgumentList '--smoke-test "OUTPUT_FOLDER"' -Wait -WindowStyle Hidden
```

This creates synthetic fixtures and results in the supplied output folder. It does not upload documents.
