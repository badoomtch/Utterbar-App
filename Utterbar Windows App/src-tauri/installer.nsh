; NSIS hooks for Utterbar.
;
; Tauri places bundle.resources entries under $INSTDIR\resources\. Windows only searches
; $INSTDIR itself for an exe's load-time DLL imports, so we relocate the sherpa-onnx
; runtime DLLs next to utterbar.exe on install and clean them up on uninstall.

!macro NSIS_HOOK_POSTINSTALL
  SetOutPath "$INSTDIR"
  CopyFiles /SILENT "$INSTDIR\resources\sherpa-onnx-c-api.dll" "$INSTDIR"
  CopyFiles /SILENT "$INSTDIR\resources\sherpa-onnx-cxx-api.dll" "$INSTDIR"
  CopyFiles /SILENT "$INSTDIR\resources\onnxruntime.dll" "$INSTDIR"
  CopyFiles /SILENT "$INSTDIR\resources\onnxruntime_providers_shared.dll" "$INSTDIR"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\sherpa-onnx-c-api.dll"
  Delete "$INSTDIR\sherpa-onnx-cxx-api.dll"
  Delete "$INSTDIR\onnxruntime.dll"
  Delete "$INSTDIR\onnxruntime_providers_shared.dll"
!macroend
