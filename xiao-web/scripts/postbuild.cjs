// Contains AI-generated content.
// This script parses the build directory, finds all HTML files and bundle files in the immutable folder, and copies them to a dist folder, preserving the folder structure.
const fs = require('fs');
const path = require('path');

const srcDir = path.join(__dirname, '../build');
const immutableDir = path.join(srcDir, '_app', 'immutable');
const distDir = path.join(__dirname, '../dist');

function ensureDirSync(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

function copyFilePreserveStructure(srcFile, baseDir, destBase) {
  const relPath = path.relative(baseDir, srcFile);
  const destFile = path.join(destBase, relPath);
  ensureDirSync(path.dirname(destFile));
  fs.copyFileSync(srcFile, destFile);
}

function copyHtmlFiles(src, dest) {
  const files = fs.readdirSync(src, { withFileTypes: true });
  for (const file of files) {
    const fullPath = path.join(src, file.name);
    if (file.isDirectory()) {
      copyHtmlFiles(fullPath, dest);
    } else if (file.isFile() && file.name.endsWith('.html')) {
      copyFilePreserveStructure(fullPath, srcDir, dest);
    }
  }
}

function copyImmutableFiles(src, dest) {
  if (!fs.existsSync(src)) return;
  const files = fs.readdirSync(src, { withFileTypes: true });
  for (const file of files) {
    const fullPath = path.join(src, file.name);
    if (file.isDirectory()) {
      copyImmutableFiles(fullPath, dest);
    } else if (file.isFile()) {
      copyFilePreserveStructure(fullPath, immutableDir, path.join(dest, '_app', 'immutable'));
    }
  }
}

// Main copy logic
ensureDirSync(distDir);
copyHtmlFiles(srcDir, distDir);
copyImmutableFiles(immutableDir, distDir);
console.log('HTML and immutable bundle files copied to', distDir);
