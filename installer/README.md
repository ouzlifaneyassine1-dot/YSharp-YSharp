# Y# Installer

## Windows Installer (recommandé)

Téléchargez `YSharp-v8.0.1-windows-x64.exe` depuis la [page Releases](https://github.com/ouzlifaneyassine1-dot/YSharp-YSharp/releases).

Double-cliquez pour installer :
1. Choisissez le dossier d'installation (défaut : `C:\Program Files\YSharp`)
2. Cochez "Add to PATH" pour ajouter `oys` et `yo` au PATH système
3. Lancer `oys` ou `yo` depuis n'importe quel terminal

### PowerShell
```powershell
powershell -ExecutionPolicy Bypass -File installer\install.ps1
```

### npm
```cmd
npm install -g ys-lang
```

### From source
```cmd
cd compiler && cargo build --release
copy target\release\oys.exe dist\
copy target\release\yo.exe dist\
```

## Linux / macOS
```bash
curl -fsSL https://raw.githubusercontent.com/ouzlifaneyassine1-dot/YSharp-YSharp/master/scripts/install.sh | bash
```

Ou depuis les sources :
```bash
cd compiler && cargo build --release
sudo cp target/release/oys /usr/local/bin/
sudo cp target/release/yo /usr/local/bin/
```

## Build the Installer

Requires [NSIS](https://nsis.sourceforge.io/) (install via `choco install nsis`):

```cmd
cd installer
"C:\Program Files (x86)\NSIS\makensis.exe" installer.nsi
```
