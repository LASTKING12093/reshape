# Contributing to Reshape

Thanks for helping make file conversion a little less annoying.

Start with an issue for a substantial change so we can agree on the scope. Small fixes can go straight to a pull request.

## Work locally

Use Windows x64, Node.js 22 or newer, Rust with the MSVC toolchain, and Visual Studio Build Tools with the Windows SDK and C++ desktop workload. See [BUILD.md](BUILD.md) for engine preparation and packaging.

```powershell
npm ci
powershell -ExecutionPolicy Bypass -File scripts/prepare-engines.ps1
npm run tauri -- dev
```

Before submitting:

```powershell
npm run lint
npm test
npm run build
```

Follow the surrounding style. Keep unrelated formatting out of your change. For interface changes, check both English and Portuguese, keyboard navigation, and small windows. Include a screenshot or short recording using synthetic files.

Explain what changed, why, and how you tested it. Compression changes should include measured input/output sizes and checks that the output still decodes correctly. Never replace an original file just to produce a smaller result.

Do not commit personal files, engine binaries, installers, logs, credentials, or local paths. Report suspected vulnerabilities privately rather than opening a public issue.
