const fs = require('fs');
const path = require('path');

const binDir = path.join(__dirname, 'bin');
const pkgDir = __dirname;

fs.mkdirSync(binDir, { recursive: true });

// Copy binaries from the package itself
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

if (copied === 0) {
  console.error('No binaries found in package. This package is Windows-only.');
  process.exit(1);
}

console.log('Y# v8.0.1 installed successfully!');
console.log('Run: npx ys-lang build myprogram.ys');
