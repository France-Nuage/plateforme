import react from '@vitejs/plugin-react-swc';
import path from 'path';
import { defineConfig, loadEnv } from 'vite';

// Validate required environment variables
const requiredEnvVars = [
  'VITE_CONTROLPLANE_URL',
  'VITE_OIDC_CLIENT_ID',
  'VITE_OIDC_PROVIDER_NAME',
  'VITE_OIDC_PROVIDER_URL',
];

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  // Merge .env files with process.env, matching what the app sees in import.meta.env
  const env = loadEnv(mode, process.cwd(), '');

  for (const envVar of requiredEnvVars) {
    if (!env[envVar]) {
      throw new Error(`Missing required environment variable: ${envVar}`);
    }
  }

  return {
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
      allowedHosts: ['console.test'],
      host: '0.0.0.0',
      port: env.PORT ? Number(env.PORT) : 5173,
    },
  };
});
