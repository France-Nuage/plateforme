// Validate required environment variables
const requiredEnvVars = [
  "NEXT_PUBLIC_CONSOLE_URL",
  "NEXT_PUBLIC_OIDC_CLIENT_ID",
  "NEXT_PUBLIC_OIDC_PROVIDER_NAME",
  "NEXT_PUBLIC_OIDC_PROVIDER_URL",
];

for (const envVar of requiredEnvVars) {
  if (!process.env[envVar]) {
    throw new Error(`Missing required environment variable: ${envVar}`);
  }
}

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "export",
  eslint: {
    ignoreDuringBuilds: true,
    dirs: ["features", "fixtures", "hooks", "providers", "services", "types"],
  },
  trailingSlash: true,
};

export default nextConfig;
