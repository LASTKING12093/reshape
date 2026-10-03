# LibreOffice 26.2.6 artifact authentication

Observed: 2026-10-03. This authenticates the dependency; it does **not** clear the entire Reshape release.

- File: `LibreOffice_26.2.6_Win_x86-64.msi`
- Public release: **26.2.6**; extracted product build: **26.2.6.3**, commit `8221e31b3ac356a1623c672912a3d2b492f7e3d1`.
- Size: **373252096 bytes**.
- Selected HTTPS source: [ibiblio artifact](https://mirrors.ibiblio.org/libreoffice/stable/26.2.6/win/x86_64/LibreOffice_26.2.6_Win_x86-64.msi), hostname `mirrors.ibiblio.org`; no redirect was required.
- Mirror provenance: [TDF's own mirror-administration archive](https://listarchives.documentfoundation.org/www/mirrors/msg00397.html) records ibiblio's registration, scanning and enabling in the TDF mirror database. This is historical primary evidence; the current live mirror-status endpoint timed out. The current artifact's publisher is additionally authenticated below.
- Second copy: [OSU Open Source Lab](https://ftp.osuosl.org/pub/tdf/libreoffice/stable/26.2.6/win/x86_64/LibreOffice_26.2.6_Win_x86-64.msi). The complete downloaded file has the same SHA-256. This cross-check is supplementary, not the basis of key identity or an installer requirement.

## OpenPGP

The matching `.msi.asc` was downloaded beside the selected artifact. GnuPG returned exit code 0, `GOODSIG` and `VALIDSIG` for the exact MSI.

- Fingerprint: **C283 9ECA D940 8FBE 9531 C3E9 F434 A1EF AFEE AEA3**.
- Identity: `LibreOffice Build Team (CODE SIGNING KEY) <build@documentfoundation.org>`.
- RSA 4096-bit signature, SHA-256 digest.
- Signature time: **2026-09-03 15:27:11 UTC**.
- Independent key source: [Fedora's official LibreOffice packaging key](https://src.fedoraproject.org/rpms/libreoffice/raw/rawhide/f/gpgkey-C2839ECAD9408FBE9531C3E9F434A1EFAFEEAEA3.gpg.asc). Its [package specification](https://src.fedoraproject.org/rpms/libreoffice/raw/rawhide/f/libreoffice.spec) uses this key to verify TDF source signatures. The fingerprint also matches the public key returned by keys.openpgp.org. No key provided by the MSI mirror was trusted.

The isolated verification keyring has no personal web-of-trust assignments, so GnuPG reports `TRUST_UNDEFINED`. Cryptographic verification passed; publisher identity is corroborated by the independent packaging source and the separate Windows code-signing certificate.

An exploratory check against the archive's **different** `26.2.6.2` signature failed, as expected for different build bytes. That signature was rejected and was not used to authenticate or pin this 26.2.6.3 artifact. File-name similarity is not interchangeability.

## Authenticode

`Get-AuthenticodeSignature`: **Valid**, publisher **The Document Foundation** (`info@documentfoundation.org`, LibreOffice Build Team, Berlin, DE).

Windows SDK SignTool `verify /pa /all /v /tw`: **success, 1 signature, 0 warnings, 0 errors**. It verified the signing chain and timestamp chain through Certum Trusted Network CA.

- Signer certificate thumbprint: `6480532A562B36D1BFFFC5B5EACF7C31E74E9B28`.
- Verified timestamp: **2026-09-03 15:11:32 UTC**.
- Timestamp authority: **Certum Timestamp 2026**, Asseco Data Systems S.A.
- Timestamp certificate thumbprint: `571468410CA85AF3424EF9164A513610F4D38D98`.

The publisher certificate expired on September 13, after the timestamped signature. A bare current-time X509Chain check therefore returns `NotTimeValid`; Authenticode's timestamp-aware verification succeeds. The expiry was investigated rather than ignored or bypassed. No trust-store changes or disabled certificate checks were used.

## Canonical whole-file SHA-256

Calculated locally **after** OpenPGP and Authenticode verification:

`f9877032fd908beb9c0ddf06df4af5c2e85f419c42e14876c4cce5aae5fb2660`

This is the whole MSI hash, not SignTool's Authenticode image digest. Both independently downloaded copies match. `engines.lock.json` pins this value, size and release version. The installer must reject old, corrupted or future artifacts even when their filename is correct.
