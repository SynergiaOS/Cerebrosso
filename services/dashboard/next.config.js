/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    appDir: true,
  },
  env: {
    NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080',
    NEXT_PUBLIC_NINJA_URL: process.env.NEXT_PUBLIC_NINJA_URL || 'http://localhost:8081',
  },
  async rewrites() {
    return [
      {
        source: '/api/cerebro/:path*',
        destination: `${process.env.NEXT_PUBLIC_API_URL}/api/:path*`,
      },
      {
        source: '/api/ninja/:path*',
        destination: `${process.env.NEXT_PUBLIC_NINJA_URL}/api/:path*`,
      },
    ];
  },
  images: {
    domains: ['localhost'],
  },
  webpack: (config) => {
    config.resolve.fallback = {
      ...config.resolve.fallback,
      fs: false,
      net: false,
      tls: false,
    };
    return config;
  },
};

module.exports = nextConfig;
