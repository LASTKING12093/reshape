; Refresh shortcut icon sources and application identity without erasing Explorer caches.
!macro ReshapeShortcutIdentity shortcut
  !insertmacro ComHlpr_CreateInProcInstance ${CLSID_ShellLink} ${IID_IShellLink} r0 ""
  ${If} $0 P<> 0
    ${IUnknown::QueryInterface} $0 '("${IID_IPersistFile}",.r1)'
    ${If} $1 P<> 0
      ${IPersistFile::Load} $1 '("${shortcut}", ${STGM_READWRITE})'
      ${IShellLink::SetIconLocation} $0 '(w "$INSTDIR\icons\reshape-app.ico",i 0)'
      ${IPersistFile::Save} $1 '("${shortcut}",1)'
      ${IUnknown::Release} $1 ""
    ${EndIf}
    ${IUnknown::Release} $0 ""
  ${EndIf}
  !undef BUNDLEID
  !define BUNDLEID "com.reshape.desktop"
  !insertmacro SetLnkAppUserModelId "${shortcut}"
  !undef BUNDLEID
  !define BUNDLEID "com.forma.converter"
  System::Call 'shell32::SHChangeNotify(i 0x2000, i 0x1005, w "${shortcut}", p 0)'
!macroend

; Only setup-owned processing engines are removed. User files and app history live elsewhere.
!macro NSIS_HOOK_PREUNINSTALL
  ${If} ${FileExists} "$INSTDIR\binaries\.reshape-engines"
    RMDir /r "$INSTDIR\binaries"
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  Delete "$DESKTOP\Forma.lnk"
  Delete "$SMPROGRAMS\Forma.lnk"
  ${If} ${FileExists} "$DESKTOP\Reshape.lnk"
    !insertmacro ReshapeShortcutIdentity "$DESKTOP\Reshape.lnk"
  ${EndIf}
  ${If} ${FileExists} "$SMPROGRAMS\Reshape.lnk"
    !insertmacro ReshapeShortcutIdentity "$SMPROGRAMS\Reshape.lnk"
  ${EndIf}
  ; Preserve pinning and arguments; only repair links pointing to this installation.
  FindFirst $R8 $R9 "$APPDATA\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\*.lnk"
  ${DoWhile} $R9 != ""
    !insertmacro IsShortcutTarget "$APPDATA\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\$R9" "$INSTDIR\${MAINBINARYNAME}.exe"
    Pop $R7
    ${If} $R7 = 1
      !insertmacro ReshapeShortcutIdentity "$APPDATA\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\$R9"
    ${EndIf}
    FindNext $R8 $R9
  ${Loop}
  FindClose $R8
!macroend
