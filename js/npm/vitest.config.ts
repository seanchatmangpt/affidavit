import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    pool: 'forks',
    include: ['test/*.test.ts'],
    // browser.test.ts needs vitest browser mode (phase 2 workspace split).
    exclude: ['test/browser.test.ts'],
    testTimeout: 120_000,
  },
});
