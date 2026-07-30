'use strict';

const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const test = require('node:test');

const repositoryRoot = join(__dirname, '..');

function workflow(name) {
  return readFileSync(join(repositoryRoot, '.github', 'workflows', name), 'utf8');
}

test('Linux release assets use and verify an older glibc baseline', () => {
  const release = workflow('release.yml');

  assert.match(release, /image:\s+rust:1\.85-bullseye/);
  assert.match(release, /dpkg --compare-versions "\$MAX_GLIBC" le "2\.31"/);
});

test('release workflow never clobbers existing assets', () => {
  const release = workflow('release.yml');

  assert.doesNotMatch(release, /--clobber/);
  assert.match(release, /existing_assets/);
  assert.match(release, /missing_assets/);
});

test('npm publishing verifies versions without mutating package metadata', () => {
  const publish = workflow('npm-publish.yml');

  assert.doesNotMatch(publish, /\bnpm version\b/);
  assert.match(publish, /PACKAGE_VERSION/);
  assert.match(publish, /LOCK_VERSION/);
  assert.match(publish, /CARGO_VERSION/);
});
