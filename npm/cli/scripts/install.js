#!/usr/bin/env node

const https = require('https');
const http = require('http');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');
const { createReadStream, createWriteStream } = require('fs');
const { pipeline } = require('stream/promises');
const tar = require('tar');

const PLATFORM = process.platform;
const ARCH = process.arch;
const NODE_VERSION = process.version;
const VERSION = require('../package.json').version;

function getBinaryName() {
  return PLATFORM === 'win32' ? 'cai.exe' : 'cai';
}

function getDownloadUrl() {
  // Map to match release workflow asset naming
  const platform = PLATFORM === 'win32' ? 'windows' : PLATFORM === 'darwin' ? 'macos' : 'linux';
  const arch = ARCH === 'arm64' ? 'aarch64' : ARCH === 'x64' ? 'x86_64' : ARCH;
  return `https://github.com/duyet/coding-agent-insights/releases/download/v${VERSION}/cai-${platform}-${arch}.tar.gz`;
}

async function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const protocol = url.startsWith('https') ? https : http;
    const file = createWriteStream(dest);

    protocol.get(url, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        downloadFile(response.headers.location, dest)
          .then(resolve)
          .catch(reject);
        return;
      }

      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode} ${response.statusMessage}`));
        return;
      }

      response.pipe(file);

      file.on('finish', () => {
        file.close();
        resolve();
      });
    }).on('error', (err) => {
      fs.unlink(dest, () => {});
      reject(err);
    });
  });
}

async function extractTarGzip(tarPath, destDir) {
  await tar.x({
    file: tarPath,
    cwd: destDir,
    strip: 0
  });
}

async function tryCargoInstall() {
  try {
    console.log('Installing via cargo...');
    execSync('cargo install cai-cli --version ' + VERSION, {
      stdio: 'inherit',
      env: { ...process.env, CARGO_INSTALL_ROOT: path.join(__dirname, '..', 'bin') }
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

  const binaryName = getBinaryName();
  const binaryPath = path.join(binDir, binaryName);
  const tarPath = path.join(os.tmpdir(), `cai-${VERSION}.tar.gz`);

  // Try pre-built binary first
  try {
    const url = getDownloadUrl();
    console.log(`Downloading from ${url}...`);
    await downloadFile(url, tarPath);
    console.log('Extracting...');
    await extractTarGzip(tarPath, binDir);
    fs.unlinkSync(tarPath);

    // Make executable
    if (PLATFORM !== 'win32') {
      fs.chmodSync(binaryPath, 0o755);
    }
    console.log('Installed successfully!');
    return;
  } catch (err) {
    console.log('Pre-built binary not available:', err.message);
    if (fs.existsSync(tarPath)) {
      fs.unlinkSync(tarPath);
    }
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
