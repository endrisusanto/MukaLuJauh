!include "MUI2.nsh"

Name "MukaLuJauh"
OutFile "..\..\mukalujauh-windows-x86_64-installer.exe"
InstallDir "$PROGRAMFILES64\MukaLuJauh"
InstallDirRegKey HKLM "Software\MukaLuJauh" "InstallDir"
RequestExecutionLevel admin

!define MUI_ABORTWARNING

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Install"
  SetOutPath "$INSTDIR"
  File "..\..\target\release\mukalujauh.exe"
  File "..\..\README.md"
  File "..\..\LICENSE"

  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "Software\MukaLuJauh" "InstallDir" "$INSTDIR"

  # Register in Windows Add/Remove Programs
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MukaLuJauh" "DisplayName" "MukaLuJauh - Face Unlock"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MukaLuJauh" "Publisher" "Endri Susanto"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MukaLuJauh" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MukaLuJauh" "QuietUninstallString" "$INSTDIR\uninstall.exe /S"

  # Create Start Menu & Desktop Shortcuts
  CreateDirectory "$SMPROGRAMS\MukaLuJauh"
  CreateShortcut "$SMPROGRAMS\MukaLuJauh\MukaLuJauh Studio.lnk" "$INSTDIR\mukalujauh.exe" "gui"
  CreateShortcut "$SMPROGRAMS\MukaLuJauh\Uninstall.lnk" "$INSTDIR\uninstall.exe"
  CreateShortcut "$DESKTOP\MukaLuJauh.lnk" "$INSTDIR\mukalujauh.exe" "gui"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\mukalujauh.exe"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\uninstall.exe"
  Delete "$DESKTOP\MukaLuJauh.lnk"
  Delete "$SMPROGRAMS\MukaLuJauh\MukaLuJauh Studio.lnk"
  Delete "$SMPROGRAMS\MukaLuJauh\Uninstall.lnk"
  RMDir "$SMPROGRAMS\MukaLuJauh"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MukaLuJauh"
  DeleteRegKey HKLM "Software\MukaLuJauh"
SectionEnd
