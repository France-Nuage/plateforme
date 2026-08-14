import path from 'path';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  test: {
    environment: 'node',
    globals: false,
    server: {
      deps: {
        // The SDK dist uses extensionless directory imports that Node ESM
        // rejects; inlining lets Vite resolve them like the app build does.
        inline: ['@france-nuage/sdk'],
      },
    },
    setupFiles: ['./vitest.setup.ts'],
  },
});
