# Online release audit — 2.0.4 candidate

Status: **CLEARED FOR PUBLICATION** (2026-10-03).

The online distribution choice is approved. No further installer-choice approval is required. No repository or release has been published by this work.

## Previous blocked candidate — historical security gate

The previous lock recorded LibreOffice **25.8.4.2 — rejected**, which must not ship in the public candidate. The [upstream security advisories](https://www.libreoffice.org/security/) identify document-import memory-safety fixes and CVE-2026-63278 information disclosure fixed in **26.2.5**. Macro disabling alone does not address these parser vulnerabilities.

The vendor archive index confirmed `26.2.5.2/win/x86_64/LibreOffice_26.2.5.2_Win_x86-64.msi`. Attempts to retrieve its metalink, mirror list and SHA-256 endpoint timed out. That attempt did not authenticate a replacement. This historical endpoint blocker was resolved using the 26.2.6 artifact and the signature strategy documented below.

The old lock entry has now been replaced; 25.8.4.2 is retained only in historical notes and negative tests.

## Integration and distribution findings

| Component | Current pinned version | License scope | Integration | Candidate distribution |
| --- | --- | --- | --- | --- |
| FFmpeg / ffprobe | 8.1.2 Gyan essentials | GPL-3.0-or-later; linked dependencies | Separate CLI; files and stdout/stderr | Gyan's GitHub release, directly downloaded |
| ImageMagick | 7.1.2-31 Q16 HDRI | ImageMagick license; delegates | Separate CLI | ImageMagick GitHub release, directly downloaded |
| 7-Zip | 26.01 | LGPL-2.1-or-later, BSD, unRAR restriction | Separate CLI and its own DLL | ip7z GitHub release, directly downloaded |
| Pandoc | 3.12 | GPL-2.0-or-later; dependencies | Separate CLI | jgm/pandoc GitHub release, directly downloaded |
| QPDF | 12.4.2 | Apache-2.0; dependencies | Separate CLI and its own DLLs | qpdf GitHub release, directly downloaded |
| Poppler | 26.09.0-0 Windows package | GPL-2.0-or-later; dependencies | Separate CLI and its own DLLs | oschwartz10612 supplier release, directly downloaded |
| LibreOffice | 26.2.6 (26.2.6.3) | MPL-2.0; third-party licenses | Separate headless CLI and its own DLLs | Authenticated TDF mirror artifact; directly downloaded |
| 7zr | 26.01 | LGPL-2.1-or-later; BSD | Temporary extraction subprocess | ip7z GitHub release, directly downloaded |
| lessmsi | 2.12.9 | MIT; WiX MS-RL; libmspack LGPL | Temporary extraction subprocess | activescott GitHub release, directly downloaded |
| WebView2 installer | 1.3.271.7, fixed artifact in lock | Microsoft runtime terms | Installs the OS WebView2 runtime | Microsoft download, only if missing |

`engines.lock.json` records exact download URLs, sizes and SHA-256 values. The regenerated inventory records 408 executable/DLL/PYD entries and the installer verifies 20,334 extracted files. WebView2's installer file version is not the browser runtime version; installed Evergreen runtimes update independently.

The application starts engines through `std::process::Command` in `src-tauri/src/process.rs`. It passes ordinary CLI arguments, file paths and process pipes. It does not use FFmpeg, Poppler, Pandoc or LibreOffice library APIs, shared internal structures or in-process plugins. Engine DLLs are loaded by their separate child executables. The Windows Rust dependency metadata contains no GPL/AGPL dependency. This supports retaining MIT for original Reshape code; it does not relicense third-party code or waive obligations for any binary we redistribute.

The Gyan FFmpeg build enables GPL and version3 and statically includes x264/x265 inside FFmpeg. It does not enable nonfree. `docs/ffmpeg-build.txt` records the supplier build configuration. An LGPL-only FFmpeg build would omit the x264/x265 software encoders used by existing workflows, so the candidate retains the separate GPL executable instead of silently removing these capabilities. Gyan is a Windows-build provider linked by FFmpeg, not an official FFmpeg-produced binary.

Earlier offline installers included GPL FFmpeg, Pandoc and Poppler (plus independently licensed dependencies). The Tauri resource configuration now excludes **all processing-engine directories**, and excludes the WebView2 offline installer. The proposed public bootstrapper downloads these packages directly from the suppliers; no Reshape proxy or engine release assets are planned. Local caches and previous offline artifacts remain development-only and ignored by Git. The final NSIS payload was inspected and contains no processing engines. Git and source scans are release checks recorded below.

Processing is designed to remain local; setup contacts suppliers. Do not claim that the entire installation never accesses the internet, or that network isolation of every document parser has been proven by these tests.


## Replacement verification — 2026-10-03

**LibreOffice 26.2.6 — verified artifact and functional engine.** See [authentication evidence](LIBREOFFICE_AUTHENTICATION.md) for exact source, file, size, SHA-256, PGP identity/fingerprint and Authenticode timestamp verification. The public release number is 26.2.6 and the actual product build is 26.2.6.3. Both downloaded copies have identical SHA-256. The release policy requires this exact version, size and hash; future versions are not automatically accepted.

The extracted-file inventory was regenerated from the authenticated clean engine tree: **20,334 files**. The binary inventory contains **408 executable/DLL/PYD records** from the real development engine tree. No old inventory was relabeled.

Current validation:

- 60 application checks passed (the previous 58 plus two office suites). The office suite covers DOCX, ODT, RTF, PPTX and XLSX to readable PDF, plausible page counts, unchanged input and temporary cleanup.
- Invalid, truncated, falsely named and corrupted documents fail safely. Office subprocess execution has a five-minute limit; this is process lifecycle control, not a security sandbox claim.
- 30 installer checks passed, including old cache rejection, exact lock policy, corrupt/wrong hash, truncation, cancellation/retry, unknown size, simulated full disk, extraction failure and directory-commit rollback.
- Empty-directory engine preparation downloaded all required suppliers' packages, extracted and verified successfully. Download total with WebView2 already installed: **611,727,975 bytes**. WebView2 adds **212,272,848 bytes** when required.
- Production core build and TypeScript passed. The candidate bootstrapper is approximately 7.3 MB. NSIS payload inspection found only the Reshape executable, icon, legal texts and NSIS plugins; no processing engine or WebView2 offline package.
- npm and RustSec: zero known vulnerabilities. RustSec's proc-macro-error/glib warnings were checked and neither is in the Windows target graph.
- Privacy scan: zero findings. Gitleaks: zero actual secrets; initial detections were checked as generated SHA-256 entries. The narrow allowlist requires both the inventory path and a complete JSON SHA-256 line, and does not exclude general source or legal files.

A fresh silent installation completed with exit code 0. The installed production application passed all **18 smoke workflows**, with **8 engine readiness/integrity checks**. These include office-to-PDF, FLAC compression, low-bitrate video compression, original preservation, previews and queue recovery.

The installer preview was inspected earlier. Further visual progress inspection was unavailable because OS automation rejected focus restoration after user input (“user input was detected”). This is an automation limitation, not an installation failure. Automated tests validate byte progress, cancellation, SHA-256, retry and cleanup. The release owner explicitly accepted this visual limitation.

The release does not claim testing on every clean Windows VM. The Reshape installer is unsigned. No automatic app updater is provided. Known limitations remain documented separately. Publication and download-back results will be recorded after the release exists.

## Final publication gate

Production build and payload inspection passed. The final installer supports fresh installation and verified-engine reuse; silent reinstallation completed with exit code 0. Source privacy scan: 132 files, zero findings. Gitleaks source and complete Git history scans: zero findings. Historical blobs also passed the personal-path/machine-name scan. The full dependency notices include upstream texts for packages without packaged license files. No known blocking vulnerability, secret, private file, unauthenticated engine or redistribution issue was identified for this online artifact.

Only the Reshape bootstrapper and its checksum are approved release assets; offline engine bundles remain excluded. The release owner accepted the visual automation limitation. GitHub CI and download-back validation follow publication and are recorded separately.
