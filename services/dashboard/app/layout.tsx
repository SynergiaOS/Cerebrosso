import './globals.css'
import { Inter, JetBrains_Mono } from 'next/font/google'
import { Providers } from './providers'

const inter = Inter({
  subsets: ['latin'],
  variable: '--font-inter',
})

const jetbrainsMono = JetBrains_Mono({
  subsets: ['latin'],
  variable: '--font-mono',
})

export const metadata = {
  title: '🥷 Cerberus Phoenix v2.0 - Solana HFT Ninja',
  description: 'Ultra Dashboard PWA for Solana High-Frequency Trading with Multi-RPC optimization',
  keywords: ['solana', 'hft', 'trading', 'multi-rpc', 'pwa', 'dashboard'],
  authors: [{ name: 'SynergiaOS', url: 'https://github.com/SynergiaOS' }],
  creator: 'SynergiaOS',
  publisher: 'Cerberus Phoenix Project',
  robots: 'noindex, nofollow', // Private dashboard
  viewport: 'width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no',
  themeColor: '#0f172a',
  manifest: '/manifest.json',
  appleWebApp: {
    capable: true,
    statusBarStyle: 'black-translucent',
    title: 'Cerberus Phoenix',
  },
  icons: {
    icon: '/icons/icon-192x192.png',
    apple: '/icons/icon-192x192.png',
  },
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en" className={`${inter.variable} ${jetbrainsMono.variable} dark`}>
      <head>
        <meta name="application-name" content="Cerberus Phoenix" />
        <meta name="apple-mobile-web-app-capable" content="yes" />
        <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
        <meta name="apple-mobile-web-app-title" content="Cerberus Phoenix" />
        <meta name="format-detection" content="telephone=no" />
        <meta name="mobile-web-app-capable" content="yes" />
        <meta name="msapplication-TileColor" content="#0f172a" />
        <meta name="msapplication-tap-highlight" content="no" />

        <link rel="apple-touch-icon" href="/icons/icon-192x192.png" />
        <link rel="icon" type="image/png" sizes="32x32" href="/icons/icon-32x32.png" />
        <link rel="icon" type="image/png" sizes="16x16" href="/icons/icon-16x16.png" />
        <link rel="manifest" href="/manifest.json" />
        <link rel="shortcut icon" href="/favicon.ico" />
      </head>
      <body className={`${inter.className} bg-slate-950 text-white antialiased overflow-x-hidden`}>
        <Providers>
          <div className="min-h-screen bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
            {children}
          </div>
        </Providers>
      </body>
    </html>
  )
}
