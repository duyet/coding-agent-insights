#!/usr/bin/env node

const https = require('https');
const http = require('http');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

const PLATFORM = process.platform;
const ARCH = process.arch;
const NODE_VERSION = process.version;
const VERSION = require('../package.json').version;

// Map architectures
const ARCH_MAP = {
  'x64': 'x86_64',
  'arm64': 'aarch64',
  'ia32': 'i686'
};

const PLATFORM_MAP = {
  'darwin': 'apple-darwin',
  'linux': 'unknown-linux-gnu',
  'win32': 'pc-windows-msvc'
};

function getTargetTriple() {
  const arch = ARCH_MAP[ARCH] || ARCH;
  const platform = PLATFORM_MAP[PLATFORM] || PLATFORM;
  return `${arch}-${platform}`;
}

function getBinaryName() {
  return PLATFORM === 'win32' ? 'cai.exe' : 'cai';
}

function getBinaryUrl() {
  const triple = getTargetTriple();
  const name = getBinaryName();
  return `https://github.com/yourusername/coding-agent-insights/releases/download/v${VERSION}/cai-${triple}`;
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const protocol = url.startsWith('https') ? https : http;
    const file = fs.createWriteStream(dest);

    protocol.get(url, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        downloadFile(response.headers.location, dest)
          .then(resolve)
          .catch(reject);
        return;
      }

      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode}`));
        return;
      }

      response.pipe(file);

      file.on('finish', () => {
        file.close();
        fs.chmodSync(dest, 0o755);
        resolve();
      });
    }).on('error', (err) => {
      fs.unlink(dest, () => {});
      reject(err);
    });
  });
}

async function tryCargoInstall() {
  try {
    console.log('Installing via cargo...');
    execSync('cargo install cai-cli --version ' + VERSION, {
      stdio: 'inherit'
    });
    return true;
  } catch (err) {
    return false;
  }
}

async function main() {
  console.log(`Installing CAI CLI v${VERSION}...`);
  console.log(`Platform: ${PLATFORM} ${ARCH}`);
  console.log(`Node: ${NODE_VERSION}`);

  const binDir = path.join(__dirname, '..', 'bin');
  fs.mkdirSync(binDir, { recursive: true });

  const binaryPath = path.join(binDir, getBinaryName());

  // Try pre-built binary first
  try {
    const url = getBinaryUrl();
    console.log(`Downloading from ${url}...`);
    await downloadFile(url, binaryPath);
    console.log('Installed successfully!');
    return;
  } catch (err) {
    console.log('Pre-built binary not available:', err.message);
  }

  // Fallback to cargo install
  if (await tryCargoInstall()) {
    console.log('Installed via cargo!');
    return;
  }

  console.error('Failed to install CAI CLI');
  console.error('Please install manually:');
  console.error('  cargo install cai-cli');
  process.exit(1);
}

main().catch(err => {
  console.error('Installation failed:', err);
  process.exit(1);
});
