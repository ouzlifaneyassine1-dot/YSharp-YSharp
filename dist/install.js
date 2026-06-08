const fs = require('fs');
const path = require('path');

const binDir = path.join(__dirname, 'bin');
const pkgDir = __dirname;

fs.mkdirSync(binDir, { recursive: true });

const binaries = ['oys.exe', 'yo.exe'];
let copied = 0;
for (const bin of binaries) {
  const src = path.join(pkgDir, bin);
  const dst = path.join(binDir, bin);
  if (fs.existsSync(src)) {
    fs.copyFileSync(src, dst);
    copied++;
    console.log(`  Installed ${bin}`);
  }
}

const extras = ['install.ps1', 'uninstall.ps1', 'y-sharp-v8.0.5.vsix'];
for (const f of extras) {
  const src = path.join(pkgDir, f);
  const dst = path.join(binDir, f);
  if (fs.existsSync(src)) {
    fs.copyFileSync(src, dst);
  }
}

if (copied === 0) {
  console.error('No binaries found in package. This package is Windows-only.');
  process.exit(1);
}

console.log('Y# v9.0.1 installed successfully via npm!');
console.log('Run: npx ys-lang build myprogram.ys');
console.log('For full Windows installation, run: install.ps1');
