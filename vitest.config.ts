import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'jsdom',
    globals: true,
    include: ['apps/desktop/src/**/*.test.{ts,tsx}'],
  },
});
