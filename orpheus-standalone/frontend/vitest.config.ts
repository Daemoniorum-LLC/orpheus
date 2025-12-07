import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./vitest.setup.ts', './src/setupTests.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html', 'lcov'],
      all: true,
      include: ['packages/*/src/**/*.{ts,tsx}', 'src/**/*.{ts,tsx}'],
      exclude: [
        '**/*.d.ts',
        '**/*.config.*',
        '**/node_modules/**',
        '**/dist/**',
        '**/build/**',
        '**/*.test.{ts,tsx}',
        '**/*.spec.{ts,tsx}',
        '**/tests/**',
        '**/__tests__/**',
        'src/main.tsx',
        'src/vite-env.d.ts',
      ],
      thresholds: {
        lines: 100,
        functions: 100,
        branches: 100,
        statements: 100,
      },
    },
  },
  resolve: {
    alias: {
      '@maestro-ai/shared-types': path.resolve(__dirname, './packages/shared-types/src'),
      '@maestro-ai/audio-analysis': path.resolve(__dirname, './packages/audio-analysis/src'),
      '@maestro-ai/guitar-pro-parser': path.resolve(__dirname, './packages/guitar-pro-parser/src'),
      '@maestro-ai/midi-utils': path.resolve(__dirname, './packages/midi-utils/src'),
      '@maestro-ai/music-theory': path.resolve(__dirname, './packages/music-theory/src'),
      '@maestro-ai/project-model': path.resolve(__dirname, './packages/project-model/src'),
      '@maestro-ai/timeline-sync': path.resolve(__dirname, './packages/timeline-sync/src'),
    },
  },
});
