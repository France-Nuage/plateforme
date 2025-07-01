import react from '@vitejs/plugin-react-swc';
import path from 'path';
import { defineConfig } from 'vite';

// Validate required environment variables
const requiredEnvVars = [
  'VITE_CONTROLPLANE_URL',
  'VITE_OIDC_CLIENT_ID',
  'VITE_OIDC_PROVIDER_NAME',
  'VITE_OIDC_PROVIDER_URL',
];

for (const envVar of requiredEnvVars) {
  if (!process.env[envVar]) {
    throw new Error(`Missing required environment variable: ${envVar}`);
  }
}

// https://vite.dev/config/
export default defineConfig({
  build: {
    sourcemap: true,
  },
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    allowedHosts: ['console'],
    host: '0.0.0.0',
    port: process.env.PORT ? Number(process.env.PORT) : 5173,
  },
});
