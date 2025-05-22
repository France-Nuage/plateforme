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
