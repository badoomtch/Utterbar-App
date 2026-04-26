; NSIS hooks for Utterbar.
;
; Tauri places bundle.resources entries under $INSTDIR\resources\. Windows only searches
; $INSTDIR itself for an exe's load-time DLL imports, so we relocate the sherpa-onnx
; runtime DLLs next to utterbar.exe on install and clean them up on uninstall.
;
; Silent-mode auto-relaunch (v0.8.5):
; When the installer was invoked with /S (the in-app updater path), NSIS itself
; re-launches Utterbar after the install completes. This is how Squirrel /
; VS Code / similar tools do it: the relaunch happens inside the installer
; process, which is the natural parent for a detached child. Earlier
; PowerShell-based watchers (v0.7.5–v0.8.3) failed because they ran outside
; the install process and tripped over Windows path quirks, antivirus
; intercepts, and timing assumptions about NSIS being done.
;
; ExecShell uses Windows' ShellExecute which:
;   - Does NOT wait for the child to finish
;   - Is fully detached: the child outlives the installer's exit
;   - Goes through the shell, so SmartScreen/ACL/etc. handle it the same
;     way as a manual user double-click — no special privilege required
;   - Has no console attached, no command-line parsing of paths
; The empty verb "" is "default verb for the file type" which for an .exe
; is "open" (run it).

!macro NSIS_HOOK_POSTINSTALL
  SetOutPath "$INSTDIR"
  CopyFiles /SILENT "$INSTDIR\resources\sherpa-onnx-c-api.dll" "$INSTDIR"
  CopyFiles /SILENT "$INSTDIR\resources\sherpa-onnx-cxx-api.dll" "$INSTDIR"
  CopyFiles /SILENT "$INSTDIR\resources\onnxruntime.dll" "$INSTDIR"
  CopyFiles /SILENT "$INSTDIR\resources\onnxruntime_providers_shared.dll" "$INSTDIR"

  ; Auto-relaunch when invoked silently (i.e. by the in-app updater).
  ; Manual installer runs (user double-clicked the .exe) skip this so
  ; we don't surprise-launch the app on a first install or a manual
  ; update.
  ;
  ; `IfSilent` is a built-in NSIS instruction taking two relative jump
  ; targets: jump-if-silent, jump-if-not-silent. `+1` means "next
  ; instruction" (i.e. fall through). The pattern below: if silent,
  ; fall through to ExecShell; if not silent, jump past it to the
  ; skip label. We avoid LogicLib's `${IfSilent}` because Tauri's
  ; NSIS template doesn't auto-include LogicLib in hook scripts.
  IfSilent +1 utterbar_skip_relaunch
  ExecShell "" "$INSTDIR\utterbar.exe"
  utterbar_skip_relaunch:
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\sherpa-onnx-c-api.dll"
  Delete "$INSTDIR\sherpa-onnx-cxx-api.dll"
  Delete "$INSTDIR\onnxruntime.dll"
  Delete "$INSTDIR\onnxruntime_providers_shared.dll"
!macroend
