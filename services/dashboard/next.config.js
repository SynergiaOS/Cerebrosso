/** @type {import('next').NextConfig} */
const withPWA = require('next-pwa')({
  dest: 'public',
  register: true,
  skipWaiting: true,
  disable: process.env.NODE_ENV === 'development',
})

const nextConfig = {
  experimental: {
    appDir: true,
  },
  env: {
    NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000',
    NEXT_PUBLIC_NINJA_URL: process.env.NEXT_PUBLIC_NINJA_URL || 'http://localhost:8090',
    NEXT_PUBLIC_DASHBOARD_URL: process.env.NEXT_PUBLIC_DASHBOARD_URL || 'http://localhost:3002',
  },
  async rewrites() {
    return [
      {
        source: '/api/cerebro/:path*',
        destination: `${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000'}/api/:path*`,
      },
      {
        source: '/api/ninja/:path*',
        destination: `${process.env.NEXT_PUBLIC_NINJA_URL || 'http://localhost:8090'}/api/:path*`,
      },
    ];
  },
  images: {
    domains: ['localhost', 'api.helius.xyz', 'solana-mainnet.g.alchemy.com'],
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

module.exports = withPWA(nextConfig);
