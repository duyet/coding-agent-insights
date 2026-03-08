#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const VERSION = require('../package.json').version;
const BIN_DIR = path.join(__dirname, '..', 'bin');
const BINARY_NAME = process.platform === 'win32' ? 'cai.exe' : 'cai';
const BINARY_PATH = path.join(BIN_DIR, BINARY_NAME);

function findBinary() {
  // Check local bin directory first (from npm install)
  if (fs.existsSync(BINARY_PATH)) {
    return BINARY_PATH;
  }

  // Check if cai is in PATH (from cargo install or manual install)
  const { PATH } = process.env;
  if (PATH) {
    const paths = PATH.split(path.delimiter);
    for (const p of paths) {
      const binPath = path.join(p, BINARY_NAME);
      if (fs.existsSync(binPath)) {
        return binPath;
      }
    }
  }

  return null;
}

const binary = findBinary();

if (!binary) {
  console.error('CAI binary not found!');
  console.error('Please run: npm install @cai/cli');
  console.error('Or install manually: cargo install cai-cli');
  process.exit(1);
}

// Execute binary with args
const args = process.argv.slice(2);
const proc = spawn(binary, args, {
  stdio: 'inherit',
  env: { ...process.env, CAI_NPM_VERSION: VERSION }
});

proc.on('exit', (code) => {
  process.exit(code ?? 0);
});

proc.on('error', (err) => {
  console.error('Failed to execute CAI:', err.message);
  console.error('Binary path:', binary);
  process.exit(1);
});
