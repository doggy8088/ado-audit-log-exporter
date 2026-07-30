'use strict';

const assert = require('node:assert/strict');
const { mkdtempSync, writeFileSync } = require('node:fs');
const { tmpdir } = require('node:os');
const { join } = require('node:path');
const test = require('node:test');

const {
  artifactName,
  cargoTarget,
  detectGlibcVersion,
  detectLinuxLibc,
  glibcVersionAtLeast,
  platformKey,
  releaseBaseUrl,
  sha256,
  verifyChecksum,
} = require('../npm/postinstall.cjs');

test('maps supported platforms to Rust targets', () => {
  assert.equal(platformKey('darwin', 'arm64'), 'darwin-arm64');
  assert.equal(cargoTarget('darwin', 'arm64'), 'aarch64-apple-darwin');
  assert.equal(cargoTarget('darwin', 'x64'), 'x86_64-apple-darwin');
  assert.equal(
    cargoTarget('linux', 'arm64', 'glibc', '2.31'),
    'aarch64-unknown-linux-gnu',
  );
  assert.equal(
    cargoTarget('linux', 'x64', 'glibc', '2.36'),
    'x86_64-unknown-linux-gnu',
  );
  assert.equal(cargoTarget('win32', 'x64'), 'x86_64-pc-windows-msvc');
});

test('rejects unsupported platforms', () => {
  assert.throws(
    () => cargoTarget('linux', 'arm', 'glibc', '2.31'),
    /Unsupported platform/,
  );
});

test('detects glibc and musl reports', () => {
  assert.equal(
    detectLinuxLibc({ header: { glibcVersionRuntime: '2.36' }, sharedObjects: [] }),
    'glibc',
  );
  assert.equal(
    detectLinuxLibc({
      header: {},
      sharedObjects: ['/lib/ld-musl-x86_64.so.1', '/lib/libc.musl-x86_64.so.1'],
    }),
    'musl',
  );
  assert.equal(detectLinuxLibc({ header: {}, sharedObjects: [] }), 'unknown');
  assert.equal(
    detectGlibcVersion({ header: { glibcVersionRuntime: '2.36' } }),
    '2.36',
  );
});

test('rejects musl before selecting a GNU Linux asset', () => {
  assert.throws(
    () => cargoTarget('linux', 'x64', 'musl'),
    /Unsupported Linux libc: musl/,
  );
});

test('rejects glibc older than the release asset baseline', () => {
  assert.equal(glibcVersionAtLeast('2.31'), true);
  assert.equal(glibcVersionAtLeast('2.36'), true);
  assert.equal(glibcVersionAtLeast('2.30'), false);
  assert.equal(glibcVersionAtLeast(undefined), false);
  assert.throws(
    () => cargoTarget('linux', 'x64', 'glibc', '2.30'),
    /Unsupported GNU libc version: 2.30/,
  );
});

test('formats artifact names and release URLs', () => {
  assert.equal(artifactName('x86_64-unknown-linux-gnu'), 'ado-audit-log-exporter-x86_64-unknown-linux-gnu.tar.xz');
  assert.equal(artifactName('x86_64-pc-windows-msvc'), 'ado-audit-log-exporter-x86_64-pc-windows-msvc.zip');
  assert.equal(releaseBaseUrl('1.2.3'), 'https://github.com/doggy8088/ado-audit-log-exporter/releases/download/v1.2.3');
});

test('verifies sha256 checksums', () => {
  const dir = mkdtempSync(join(tmpdir(), 'ado-audit-log-exporter-'));
  const file = join(dir, 'sample.txt');
  writeFileSync(file, 'hello');
  const digest = sha256(file);
  verifyChecksum(file, `${digest}  sample.txt`);
  assert.throws(() => verifyChecksum(file, '0'.repeat(64)), /Checksum mismatch/);
});
