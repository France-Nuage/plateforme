/** @type {import('next').NextConfig} */
const nextConfig = {
  eslint: {
    ignoreDuringBuilds: true,
    extends: ["next", "prettier"],
    dirs: ["features", "fixtures", "hooks", "providers", "services", "types"],
  },
  trailingSlash: true,
  reactStrictMode: true,
};

export default nextConfig;
