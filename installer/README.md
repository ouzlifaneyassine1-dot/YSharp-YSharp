# Y# Installer

## Windows

### Option 1: PowerShell (recommandé, sans outils supplémentaires)
```powershell
powershell -ExecutionPolicy Bypass -File installer\install.ps1
```
Installe oys.exe et yo.exe dans `C:\Program Files\YSharp\bin` et les ajoute au PATH.

### Option 2: MSI (nécessite WiX Toolset)
```cmd
cd installer
build-msi.bat
```
Produit `dist\YSharp-v8.0.1-windows-x64.msi`.

Pour installer WiX : `choco install wixtoolset`

### Option 3: npm
```cmd
npm install -g ys-lang
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
