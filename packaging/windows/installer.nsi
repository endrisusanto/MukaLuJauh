!include "MUI2.nsh"

Name "MukaLuJauh"
OutFile "mukalujauh-windows-x86_64-installer.exe"
InstallDir "$PROGRAMFILES64\MukaLuJauh"
InstallDirRegKey HKLM "Software\MukaLuJauh" "InstallDir"
RequestExecutionLevel admin

!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\orange-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\orange-uninstall.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Install"
  SetOutPath "$INSTDIR"
  File "target\release\mukalujauh.exe"
  File "README.md"
  File "LICENSE"

  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "Software\MukaLuJauh" "InstallDir" "$INSTDIR"

  # Register in Windows Add/Remove Programs
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MukaLuJauh" "DisplayName" "MukaLuJauh - Face Unlock"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MukaLuJauh" "DisplayVersion" "0.1.2"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MukaLuJauh" "Publisher" "Endri Susanto"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MukaLuJauh" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MukaLuJauh" "QuietUninstallString" "$INSTDIR\uninstall.exe /S"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\mukalujauh.exe"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MukaLuJauh"
  DeleteRegKey HKLM "Software\MukaLuJauh"
SectionEnd
