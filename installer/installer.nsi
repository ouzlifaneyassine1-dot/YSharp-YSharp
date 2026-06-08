; Y# (YSharp) v9.0.1 Installer — NSIS
; All-in-one: compiler, package manager, IDE, launcher, VS Code extension

Unicode True
!define PRODUCT_NAME "Y# (YSharp)"
!define PRODUCT_VERSION "9.0.1"
!define PRODUCT_PUBLISHER "Y# Language Team"
!define PRODUCT_WEB_SITE "https://github.com/ouzlifaneyassine1-dot/YSharp-YSharp"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\oys.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKLM"

RequestExecutionLevel admin
SetCompressor lzma

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "TextFunc.nsh"

; --- Modern UI 2 settings ---
!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TITLE "Welcome to Y# (YSharp) v9.0.1 Setup"
!define MUI_WELCOMEPAGE_TEXT "This wizard will guide you through installing Y# (YSharp) v9.0.1.$\r$\n$\r$\nIncludes: Y# compiler (oys), package manager (yo), desktop IDE with AI agents, launcher, and VS Code extension.$\r$\n$\r$\nClick Next to continue."
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "${NSISDIR}\Contrib\Graphics\Header\nsis.bmp"
!define MUI_LICENSEPAGE_RADIOBUTTONS

; --- Pages ---
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; --- Languages ---
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "French"

; --- Component sections ---
Section "Y# Core (required)" SEC_CORE
  SectionIn RO
  SetOutPath "$INSTDIR\bin"
  File "..\dist\oys.exe"
  File "..\dist\yo.exe"
  SetOutPath "$INSTDIR\launcher"
  File "..\launcher\Y# Launcher.bat"
  File "..\launcher\Y# IDE.bat"
  SetOutPath "$INSTDIR\scripts"
  File "..\dist\install.ps1"
  File "..\dist\uninstall.ps1"
  SetOutPath "$INSTDIR\vscode"
  File "..\dist\y-sharp-v8.0.5.vsix"
  SetOutPath "$INSTDIR\ide\src"
  File "..\ide\src\index.html"
  File "..\ide\src\style.css"
  File "..\ide\src\app.js"
  File "..\ide\src\preload.js"
  SetOutPath "$INSTDIR\ide"
  File "..\ide\main.js"
  File "..\ide\package.json"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\bin\oys.exe"
  WriteRegStr HKLM "${PRODUCT_UNINST_ROOT_KEY}\${PRODUCT_UNINST_KEY}" "DisplayName" "$(^Name)"
  WriteRegStr HKLM "${PRODUCT_UNINST_ROOT_KEY}\${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "${PRODUCT_UNINST_ROOT_KEY}\${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr HKLM "${PRODUCT_UNINST_ROOT_KEY}\${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
  WriteRegStr HKLM "${PRODUCT_UNINST_ROOT_KEY}\${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  WriteRegStr HKLM "${PRODUCT_UNINST_ROOT_KEY}\${PRODUCT_UNINST_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegDWORD HKLM "${PRODUCT_UNINST_ROOT_KEY}\${PRODUCT_UNINST_KEY}" "NoModify" 1
  WriteRegDWORD HKLM "${PRODUCT_UNINST_ROOT_KEY}\${PRODUCT_UNINST_KEY}" "NoRepair" 1
SectionEnd

Section "Add to PATH (system-wide)" SEC_PATH
  Push "$INSTDIR\bin"
  Call AddToPath
SectionEnd

Section "Start Menu Shortcuts" SEC_SHORTCUTS
  CreateDirectory "$SMPROGRAMS\Y# (YSharp)"
  CreateShortCut "$SMPROGRAMS\Y# (YSharp)\Y# IDE.lnk" "$INSTDIR\launcher\Y# IDE.bat" "" "$INSTDIR\bin\oys.exe" 0
  CreateShortCut "$SMPROGRAMS\Y# (YSharp)\Y# Command Prompt.lnk" "$WINDIR\System32\cmd.exe" '/K oys' "$INSTDIR\bin\oys.exe" 0
  CreateShortCut "$SMPROGRAMS\Y# (YSharp)\Uninstall Y#.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\uninstall.exe" 0
SectionEnd

Section "Associate .ys/.yse files" SEC_ASSOC
  WriteRegStr HKCU "Software\Classes\.ys" "" "YSharp.File"
  WriteRegStr HKCU "Software\Classes\.yse" "" "YSharp.File"
  WriteRegStr HKCU "Software\Classes\YSharp.File\shell\open\command" "" '"$INSTDIR\launcher\Y# Launcher.bat" "%1"'
  WriteRegStr HKCU "Software\Classes\YSharp.File\DefaultIcon" "" "$INSTDIR\bin\oys.exe,0"
  System::Call 'shell32.dll::SHChangeNotify(i, i, i, i) v (0x08000000, 0, 0, 0)'
SectionEnd

; --- Section descriptions ---
!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC_CORE} "Y# compiler (oys), package manager (yo), IDE, launcher, VS Code extension, and PowerShell scripts."
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC_PATH} "Add Y# binaries to system PATH so you can run oys/yo from any terminal."
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC_SHORTCUTS} "Start Menu shortcuts for Y# IDE, Command Prompt, and Uninstall."
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC_ASSOC} "Double-click .ys/.yse files to build and run with Y# Launcher."
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; --- Installer properties ---
Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "..\dist\YSharp-v${PRODUCT_VERSION}-windows-x64.exe"
InstallDir "$PROGRAMFILES64\YSharp"
InstallDirRegKey HKLM "${PRODUCT_DIR_REGKEY}" ""
ShowInstDetails show
ShowUnInstDetails show

; --- Uninstaller ---
Section Uninstall
  Delete "$INSTDIR\uninstall.exe"
  Delete "$INSTDIR\bin\oys.exe"
  Delete "$INSTDIR\bin\yo.exe"
  RMDir "$INSTDIR\bin"
  Delete "$INSTDIR\launcher\Y# Launcher.bat"
  Delete "$INSTDIR\launcher\Y# IDE.bat"
  RMDir "$INSTDIR\launcher"
  Delete "$INSTDIR\scripts\install.ps1"
  Delete "$INSTDIR\scripts\uninstall.ps1"
  RMDir "$INSTDIR\scripts"
  Delete "$INSTDIR\vscode\y-sharp-v8.0.5.vsix"
  RMDir "$INSTDIR\vscode"
  Delete "$INSTDIR\ide\src\index.html"
  Delete "$INSTDIR\ide\src\style.css"
  Delete "$INSTDIR\ide\src\app.js"
  Delete "$INSTDIR\ide\src\preload.js"
  RMDir "$INSTDIR\ide\src"
  Delete "$INSTDIR\ide\main.js"
  Delete "$INSTDIR\ide\package.json"
  RMDir "$INSTDIR\ide"
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\Y# (YSharp)\Y# IDE.lnk"
  Delete "$SMPROGRAMS\Y# (YSharp)\Y# Command Prompt.lnk"
  Delete "$SMPROGRAMS\Y# (YSharp)\Uninstall Y#.lnk"
  RMDir "$SMPROGRAMS\Y# (YSharp)"

  Push "$INSTDIR\bin"
  Call un.RemoveFromPath

  DeleteRegKey HKCU "Software\Classes\.ys"
  DeleteRegKey HKCU "Software\Classes\.yse"
  DeleteRegKey HKCU "Software\Classes\YSharp.File"

  DeleteRegKey HKLM "${PRODUCT_UNINST_ROOT_KEY}\${PRODUCT_UNINST_KEY}"
  DeleteRegKey HKLM "${PRODUCT_DIR_REGKEY}"
SectionEnd

; --- PATH manipulation functions ---
; AddToPath - adds a directory to the system PATH (if not already present)
Function AddToPath
  Exch $0    ; dir to add
  Push $1
  Push $2
  Push $3

  ReadRegStr $1 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "PATH"
  StrCpy $2 ";"
  StrCpy $3 $1

  ; Check if already in PATH
  Push "$1"
  Push "$2"
  Call StrStr
  Pop $1
  StrCmp $1 "" 0 done

  ; Append to PATH
  ReadRegStr $1 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "PATH"
  StrCpy $1 "$1;$3"
  WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "PATH" "$1"
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000

done:
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

Function un.RemoveFromPath
  Exch $0
  Push $1
  Push $2
  Push $3

  ReadRegStr $1 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "PATH"
  StrCpy $2 ";"
  StrCpy $3 ";$0;"

  Push "$3"
  Push "$1"
  Call un.StrReplace
  Pop $1

  WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "PATH" "$1"
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000

  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

; StrStr - finds substring (needle) in string (haystack)
Function StrStr
  Exch $0 ; needle
  Exch
  Exch $1 ; haystack
  Push $2
  Push $3
  Push $4
  Push $5
  StrCpy $2 0
  StrLen $3 $0
  IntOp $5 $3 - 1
loop:
  StrCpy $4 $1 $3 $2
  StrCmp $4 $0 found
  StrCmp $4 "" notfound
  IntOp $2 $2 + 1
  Goto loop
found:
  StrCpy $0 $1 "" $2
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Exch $0
  Goto end
notfound:
  StrCpy $0 ""
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $1
  Exch $0
end:
FunctionEnd

Function un.StrStr
  Exch $0
  Exch
  Exch $1
  Push $2
  Push $3
  Push $4
  Push $5
  StrCpy $2 0
  StrLen $3 $0
  IntOp $5 $3 - 1
loop_un:
  StrCpy $4 $1 $3 $2
  StrCmp $4 $0 found_un
  StrCmp $4 "" notfound_un
  IntOp $2 $2 + 1
  Goto loop_un
found_un:
  StrCpy $0 $1 "" $2
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Exch $0
  Goto end_un
notfound_un:
  StrCpy $0 ""
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $1
  Exch $0
end_un:
FunctionEnd

; StrReplace - replaces all occurrences of substring
Function un.StrReplace
  Exch $0 ; needle (with delimiters)
  Exch
  Exch $1 ; haystack
  Push $2
  Push $3
  Push $4
  Push $5
  Push $6
  StrCpy $2 ""
  StrLen $3 $0
  StrCpy $5 0
loop_sr:
  StrCpy $4 $1 $3 $5
  StrCmp $4 $0 found_sr
  StrCmp $4 "" done_sr
  StrCpy $6 $1 1 $5
  StrCpy $2 "$2$6"
  IntOp $5 $5 + 1
  Goto loop_sr
found_sr:
  IntOp $5 $5 + $3
  Goto loop_sr
done_sr:
  StrCpy $0 $2
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Exch $0
FunctionEnd
