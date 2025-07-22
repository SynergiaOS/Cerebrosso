'use client'

import { ReactNode } from 'react'
import { SWRConfig } from 'swr'
import { Toaster } from 'react-hot-toast'

// 🌐 SWR Fetcher function
const fetcher = async (url: string) => {
  const res = await fetch(url)
  if (!res.ok) {
    throw new Error('Failed to fetch data')
  }
  return res.json()
}

// 🎨 Toast configuration
const toastOptions = {
  duration: 4000,
  position: 'top-right' as const,
  style: {
    background: '#1e293b',
    color: '#f8fafc',
    border: '1px solid #334155',
    borderRadius: '0.75rem',
  },
  success: {
    iconTheme: {
      primary: '#22c55e',
      secondary: '#f8fafc',
    },
  },
  error: {
    iconTheme: {
      primary: '#ef4444',
      secondary: '#f8fafc',
    },
  },
}

interface ProvidersProps {
  children: ReactNode
}

export function Providers({ children }: ProvidersProps) {
  return (
    <SWRConfig
      value={{
        fetcher,
        refreshInterval: 5000, // Refresh every 5 seconds
        revalidateOnFocus: true,
        revalidateOnReconnect: true,
        dedupingInterval: 2000,
        errorRetryCount: 3,
        errorRetryInterval: 1000,
        onError: (error) => {
          console.error('SWR Error:', error)
        },
      }}
    >
      {children}
      <Toaster toastOptions={toastOptions} />
    </SWRConfig>
  )
}
