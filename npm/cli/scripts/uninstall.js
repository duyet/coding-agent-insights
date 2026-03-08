#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const binDir = path.join(__dirname, '..', 'bin');

try {
  if (fs.existsSync(binDir)) {
    fs.rmSync(binDir, { recursive: true, force: true });
    console.log('CAI CLI uninstalled successfully');
  }
} catch (err) {
  console.error('Cleanup warning:', err.message);
}
