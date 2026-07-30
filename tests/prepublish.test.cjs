'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');

const {
  expectedReleaseUrls,
  verifyReleaseAssets,
} = require('../npm/prepublish-check.cjs');

test('expects one archive and one checksum for every supported target', () => {
  const urls = expectedReleaseUrls('1.2.3');

  assert.equal(urls.length, 10);
  assert.ok(urls.every((url) => url.includes('/releases/download/v1.2.3/')));
  assert.equal(urls.filter((url) => url.endsWith('.sha256')).length, 5);
});

test('reports unavailable release assets', async () => {
  await assert.rejects(
    verifyReleaseAssets({
      version: '1.2.3',
      retries: 1,
      check: async (url) => ({ url, ok: false, statusCode: 404 }),
    }),
    /Missing or unavailable release assets for v1\.2\.3/,
  );
});
