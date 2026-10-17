// Browser-suite config. Plain vitest (node env, forks pool): the browser
// assertion happens for real inside a headless Chromium driven by
// playwright-core from within test/browser.test.ts itself, so no
// @vitest/browser / browser-mode workspace split is needed.
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    pool: 'forks',
    environment: 'node',
    include: ['test/browser.test.ts'],
    testTimeout: 120_000,
  },
});
