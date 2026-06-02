// Y# v8.0.0 platform-specific binary installer
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const https = require('https');
const os = require('os');

const pkg = require('./package.json');
const version = pkg.version;

const platformMap = {
  'win32-x64': 'windows-x64',
  'darwin-x64': 'macos-x64',
  'darwin-arm64': 'macos-arm64',
  'linux-x64': 'linux-x64',
};

const plat = `${os.platform()}-${os.arch()}`;
const distName = platformMap[plat];

if (!distName) {
  console.error(`Unsupported platform: ${plat}`);
  console.error(`Y# v${version} supports: ${Object.keys(platformMap).join(', ')}`);
  process.exit(1);
}

const url = `https://github.com/oysterlang/ys/releases/download/v${version}/ys-v${version}-${distName}.zip`;
const installDir = path.join(__dirname, 'bin');

console.log(`Installing Y# v${version} for ${plat}...`);

// Ensure bin directory
fs.mkdirSync(installDir, { recursive: true });

// Download and extract
const zipPath = path.join(__dirname, 'ys.zip');
const file = fs.createWriteStream(zipPath);

console.log(`Downloading from ${url}...`);
https.get(url, (response) => {
  if (response.statusCode !== 200) {
    console.error(`Download failed (HTTP ${response.statusCode})`);
    console.error('Please download manually from the releases page.');
    process.exit(1);
  }
  response.pipe(file);
  file.on('finish', () => {
    file.close();
    // Extract
    const AdmZip = require('adm-zip');
    const zip = new AdmZip(zipPath);
    zip.extractAllTo(installDir, true);
    fs.unlinkSync(zipPath);
    console.log('Y# installed successfully!');
    console.log('Run: npx ys-lang build myprogram.ys');
  });
}).on('error', (err) => {
  fs.unlinkSync(zipPath);
  console.error('Download failed:', err.message);
  process.exit(1);
});
