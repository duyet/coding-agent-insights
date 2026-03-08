#!/usr/bin/env node

const { execSync, spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Path to installed binary
const BIN_DIR = path.join(__dirname, '..', 'bin');
const BINARY_NAME = process.platform === 'win32' ? 'cai.exe' : 'cai';
const BINARY_PATH = path.join(BIN_DIR, BINARY_NAME);

// Check if binary exists
if (!fs.existsSync(BINARY_PATH)) {
  console.error(`CAI binary not found at ${BINARY_PATH}`);
  console.error('Please run: npm install @cai/cli');
  process.exit(1);
}

// Execute binary with args
const args = process.argv.slice(2);
const proc = spawn(BINARY_PATH, args, {
  stdio: 'inherit',
  env: { ...process.env, CAI_NPM_VERSION: require('../package.json').version }
});

proc.on('exit', (code) => {
  process.exit(code ?? 0);
});

proc.on('error', (err) => {
  console.error('Failed to execute CAI:', err.message);
  process.exit(1);
});
