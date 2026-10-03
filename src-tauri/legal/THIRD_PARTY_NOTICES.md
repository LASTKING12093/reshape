# Third-party notices

The MIT license in this repository covers Reshape's original code and branding. It does **not** relicense third-party tools or libraries.

## Application

| Project | License |
| --- | --- |
| Tauri and its official plugins | MIT OR Apache-2.0 |
| React, React DOM, Motion | MIT |
| Lucide | ISC |
| TypeScript | Apache-2.0 |
| Vite | MIT |
| Rust crates | Individual licenses recorded in their package metadata; exact versions in Cargo.lock |

The NSIS template is derived from Tauri's MIT/Apache-2.0 template. WPF uses the Windows .NET Framework runtime. Segoe UI is requested from Windows; no font file is redistributed. The Reshape mark was drawn for this project. Reference artwork is not included.

## Processing engines

| Component | Version in the local package | Distribution terms |
| --- | --- | --- |
| FFmpeg / ffprobe, Gyan essentials | 8.1.2 | GPLv3 build, with statically linked third-party libraries including x264 and x265 |
| ImageMagick | 7.1.2-31 Q16 HDRI | ImageMagick License and delegate licenses |
| LibreOffice | 26.2.6 (product build 26.2.6.3) | MPL-2.0 and third-party licenses |
| Pandoc | 3.12 | GPL-2.0-or-later and bundled library notices |
| 7-Zip | 26.01 | LGPL, BSD portions and unRAR restrictions |
| QPDF | 12.4.2 | Apache-2.0 and dependency notices |
| Poppler Windows package | 26.09.0-0 | GPL and bundled dependency licenses |
| WebView2 | Evergreen | Microsoft redistribution terms |
| Microsoft runtime DLLs | Vendor package versions | Microsoft redistribution terms |

These tools run as separate processes. That does not waive their redistribution requirements. The exact FFmpeg binary was inspected with `-buildconf`: it enables GPL, version3, static linking, x264 and x265. It is not an LGPL-only build.

## Online distribution

The online installer is the selected distribution architecture. Processing engines have been removed from the Tauri resource list and will be fetched directly from suppliers after hash verification. LibreOffice 26.2.6 is authenticated and pinned. Production payload inspection confirms no processing engines are bundled. See [the online candidate audit](docs/ONLINE_RELEASE_AUDIT.md) for the concrete integration and distribution findings.

**The current offline binary package is not cleared for public redistribution.** Complete corresponding sources, build recipes and notices for the exact GPL/LGPL components and their relevant dependencies have not all been assembled and verified. Generic upstream links are not a substitute. Do not publish the offline installer until this gate is resolved.

See [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for supplier URLs and the existing inventory. Do not interpret the project's MIT license as permission to omit third-party obligations.

