import './globals.css'
import { Inter } from 'next/font/google'
import { Providers } from './providers'

const inter = Inter({ subsets: ['latin'] })

export const metadata = {
  title: '🐺 Cerberus Phoenix v2.0 - Dashboard',
  description: 'Autonomiczny ekosystem do operacji on-chain na Solanie',
  keywords: ['solana', 'hft', 'trading', 'mev', 'jito', 'ai'],
  authors: [{ name: 'SynergiaOS', url: 'https://github.com/SynergiaOS' }],
  creator: 'SynergiaOS',
  publisher: 'Cerberus Phoenix Project',
  robots: 'noindex, nofollow', // Private dashboard
  viewport: 'width=device-width, initial-scale=1',
  themeColor: '#0ea5e9',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en" className="dark">
      <body className={`${inter.className} bg-dark-900 text-white antialiased`}>
        <Providers>
          <div className="min-h-screen bg-gradient-to-br from-dark-900 via-dark-800 to-dark-900">
            {children}
          </div>
        </Providers>
      </body>
    </html>
  )
}
