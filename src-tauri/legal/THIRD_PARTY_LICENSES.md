# Third-party licenses and distribution

Reshape original source is MIT. Third-party software retains its own license; MIT does not cover the processing engines.

The current online architecture executes FFmpeg, ImageMagick, 7-Zip, Pandoc, QPDF, Poppler and LibreOffice as separate command-line programs using files and process pipes. No GPL/AGPL engine code or engine DLL is linked into Reshape. Engine DLLs and statically linked codecs belong to those external programs.

Exact artifact versions, HTTPS supplier URLs, source links, sizes and SHA-256 values are recorded in `engines.lock.json`. `docs/engine-binary-inventory.json` records actual extracted executable/DLL identities; `docs/ffmpeg-build.txt` records the Gyan build configuration and its libraries. The full extracted-file integrity table is `installer/engine-files.json`.

See `THIRD_PARTY_NOTICES.md` for the component table. Application dependency notices and license texts are bundled as `legal/DEPENDENCY_LICENSES.txt`; original package notices remain in each downloaded engine directory. MPL-covered application dependencies are used unmodified, and their source is available from the exact packages in Cargo.lock and upstream URLs in the notices.

The public bootstrapper must not embed or host processing engines. Earlier private offline packages included GPL FFmpeg, Pandoc and Poppler; publicly distributing such binaries would require satisfying their respective source and notice obligations. Direct supplier downloads do not authorize copying those packages into a Reshape release later. An offline edition needs its own completed redistribution audit.

Fonts referenced by the interface are provided by Windows. No Windows font files are bundled in the core. Downloaded LibreOffice packages contain their own separately licensed fonts and components, whose notices must remain intact. Microsoft runtime DLLs arrive within their supplier packages; WebView2 is a Microsoft runtime with its own terms.

The concrete authentication, tests and publication status are maintained in `docs/ONLINE_RELEASE_AUDIT.md`. Do not infer release clearance from this document alone.
