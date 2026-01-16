import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@maestro-ai/shared-types': path.resolve(__dirname, '../shared-types/src'),
      '@maestro-ai/project-model': path.resolve(__dirname, '../project-model/src'),
      '@maestro-ai/music-theory': path.resolve(__dirname, '../music-theory/src'),
      '@maestro-ai/midi-utils': path.resolve(__dirname, '../midi-utils/src'),
      '@maestro-ai/timeline-sync': path.resolve(__dirname, '../timeline-sync/src'),
      '@maestro-ai/audio-analysis': path.resolve(__dirname, '../audio-analysis/src'),
      '@maestro-ai/guitar-pro-parser': path.resolve(__dirname, '../guitar-pro-parser/src'),
    },
  },
  server: {
    port: 3002,
    open: true,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          // React core
          'react-vendor': ['react', 'react-dom', 'react/jsx-runtime'],

          // Fluent UI components (large library)
          'fluent-ui': ['@fluentui/react-components', '@fluentui/react-icons'],

          // Audio libraries (large)
          'audio-libs': ['tone', '@coderline/alphatab'],

          // HTTP client
          'http-client': ['axios'],

          // State management
          'state': ['zustand'],

          // Mode components (lazy loaded, but still chunked together)
          'modes': [
            './src/modes/ComposeMode',
            './src/modes/RecordMode',
            './src/modes/MixMode',
            './src/modes/MasterMode',
            './src/modes/PracticeMode',
            './src/modes/DistributeMode',
          ],
        },
      },
    },
    chunkSizeWarningLimit: 600, // Increase threshold to 600KB
  },
});
