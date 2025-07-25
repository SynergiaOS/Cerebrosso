'use client'

import { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { 
  Activity, 
  TrendingUp, 
  DollarSign, 
  Zap,
  Shield,
  Target,
  BarChart3,
  Cpu,
  Database,
  Globe,
  Wifi,
  WifiOff,
  RefreshCw,
  Settings,
  Bell,
  Menu,
  X,
  ArrowUp,
  ArrowDown,
  Minus
} from 'lucide-react'
import { toast } from 'react-hot-toast'

// Ultra Dashboard Components
import { UltraMetricCard } from '../components/ultra/UltraMetricCard'
import { MultiRpcStatus } from '../components/ultra/MultiRpcStatus'
import { CostOptimization } from '../components/ultra/CostOptimization'
import { RealTimeEvents } from '../components/ultra/RealTimeEvents'
import { PerformanceChart } from '../components/ultra/PerformanceChart'
import { SystemHealth } from '../components/ultra/SystemHealth'
import { TradingPositions } from '../components/ultra/TradingPositions'
import { RiskAnalysis } from '../components/ultra/RiskAnalysis'

export default function UltraDashboard() {
  const [isLoading, setIsLoading] = useState(true)
  const [isOnline, setIsOnline] = useState(true)
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const [lastUpdate, setLastUpdate] = useState(new Date())
  const [systemData, setSystemData] = useState({
    costSavings: 127,
    freeRequests: 200000,
    responseTime: 127,
    successRate: 100,
    providers: {
      alchemy: { status: 'healthy' as const, usage: 45, responseTime: 127 },
      helius: { status: 'healthy' as const, usage: 30, responseTime: 156 },
      public: { status: 'healthy' as const, usage: 25, responseTime: 216 }
    }
  })

  useEffect(() => {
    // Check online status
    const handleOnline = () => {
      setIsOnline(true)
      toast.success('🌐 Connection restored')
    }
    const handleOffline = () => {
      setIsOnline(false)
      toast.error('📡 Connection lost - Using cached data')
    }

    window.addEventListener('online', handleOnline)
    window.addEventListener('offline', handleOffline)

    // Simulate loading
    setTimeout(() => {
      setIsLoading(false)
      toast.success('🥷 Cerberus Phoenix v2.0 - Ultra Dashboard Online')
    }, 2000)

    // Auto-refresh data every 30 seconds
    const interval = setInterval(() => {
      setLastUpdate(new Date())
      // Simulate data updates
      setSystemData(prev => ({
        ...prev,
        responseTime: Math.floor(Math.random() * 50) + 100,
        providers: {
          alchemy: { 
            ...prev.providers.alchemy, 
            responseTime: Math.floor(Math.random() * 50) + 100 
          },
          helius: { 
            ...prev.providers.helius, 
            responseTime: Math.floor(Math.random() * 50) + 130 
          },
          public: { 
            ...prev.providers.public, 
            responseTime: Math.floor(Math.random() * 100) + 180 
          }
        }
      }))
    }, 30000)

    return () => {
      window.removeEventListener('online', handleOnline)
      window.removeEventListener('offline', handleOffline)
      clearInterval(interval)
    }
  }, [])

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
        <motion.div className="text-center">
          <motion.div
            animate={{ rotate: 360 }}
            transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
            className="w-20 h-20 border-4 border-emerald-500 border-t-transparent rounded-full mx-auto mb-4"
          />
          <motion.h2
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="text-2xl font-bold text-white mb-2"
          >
            🥷 Cerberus Phoenix v2.0
          </motion.h2>
          <motion.p
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.5 }}
            className="text-slate-400"
          >
            Initializing Ultra Dashboard PWA...
          </motion.p>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 1 }}
            className="mt-4 text-sm text-slate-500"
          >
            Multi-RPC Optimization • FREE Providers Only
          </motion.div>
        </motion.div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* Mobile Sidebar Overlay */}
      <AnimatePresence>
        {sidebarOpen && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/50 z-40 lg:hidden"
            onClick={() => setSidebarOpen(false)}
          />
        )}
      </AnimatePresence>

      {/* Ultra Header */}
      <motion.header
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="sticky top-0 z-30 bg-slate-950/80 backdrop-blur-xl border-b border-slate-800"
      >
        <div className="flex items-center justify-between p-4">
          <div className="flex items-center space-x-4">
            <button
              onClick={() => setSidebarOpen(!sidebarOpen)}
              className="lg:hidden p-2 rounded-lg bg-slate-800 hover:bg-slate-700 transition-colors"
            >
              {sidebarOpen ? <X size={20} /> : <Menu size={20} />}
            </button>
            
            <div>
              <h1 className="text-2xl font-bold bg-gradient-to-r from-emerald-400 via-blue-500 to-purple-500 bg-clip-text text-transparent">
                🥷 Cerberus Phoenix v2.0
              </h1>
              <p className="text-slate-400 text-sm">
                Ultra Dashboard PWA • Multi-RPC Optimization • FREE Providers Only
              </p>
            </div>
          </div>

          <div className="flex items-center space-x-3">
            {/* Connection Status */}
            <motion.div 
              className="flex items-center space-x-2"
              animate={{ scale: isOnline ? 1 : 0.95 }}
              transition={{ duration: 0.2 }}
            >
              {isOnline ? (
                <Wifi className="w-5 h-5 text-emerald-500" />
              ) : (
                <WifiOff className="w-5 h-5 text-red-500" />
              )}
              <span className="text-sm text-slate-400 hidden sm:block">
                {isOnline ? 'Online' : 'Offline'}
              </span>
            </motion.div>

            {/* Last Update */}
            <div className="text-xs text-slate-500 hidden md:block">
              Updated: {lastUpdate.toLocaleTimeString()}
            </div>

            {/* Action Buttons */}
            <motion.button 
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              className="p-2 rounded-lg bg-slate-800 hover:bg-slate-700 transition-colors"
            >
              <Bell size={18} />
            </motion.button>
            <motion.button 
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              className="p-2 rounded-lg bg-slate-800 hover:bg-slate-700 transition-colors"
            >
              <Settings size={18} />
            </motion.button>
          </div>
        </div>
      </motion.header>

      <div className="p-4 lg:p-6">
        {/* Ultra Key Metrics */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6"
        >
          <UltraMetricCard
            title="Cost Savings"
            value={`$${systemData.costSavings}`}
            subtitle="/month"
            change="95% reduction"
            icon={DollarSign}
            color="emerald"
            trend="up"
            sparkline={[80, 85, 90, 95, 95]}
          />
          <UltraMetricCard
            title="Free Requests"
            value={`${(systemData.freeRequests / 1000).toFixed(0)}k+`}
            subtitle="/month"
            change="3 providers"
            icon={Globe}
            color="blue"
            trend="up"
            sparkline={[150, 170, 180, 190, 200]}
          />
          <UltraMetricCard
            title="Response Time"
            value={`${systemData.responseTime}ms`}
            subtitle="average"
            change="Alchemy optimized"
            icon={Zap}
            color="yellow"
            trend="down"
            sparkline={[180, 160, 140, 130, 127]}
          />
          <UltraMetricCard
            title="Success Rate"
            value={`${systemData.successRate}%`}
            subtitle="uptime"
            change="All providers"
            icon={Target}
            color="emerald"
            trend="up"
            sparkline={[98, 99, 99.5, 100, 100]}
          />
        </motion.div>

        {/* Ultra Dashboard Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          {/* Multi-RPC Status - Full Width */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
            className="lg:col-span-12"
          >
            <MultiRpcStatus providers={systemData.providers} />
          </motion.div>

          {/* Cost Optimization */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 }}
            className="lg:col-span-6"
          >
            <CostOptimization savings={systemData.costSavings} />
          </motion.div>

          {/* Real-time Events */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.4 }}
            className="lg:col-span-6"
          >
            <RealTimeEvents />
          </motion.div>

          {/* Performance Chart */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.5 }}
            className="lg:col-span-8"
          >
            <PerformanceChart />
          </motion.div>

          {/* System Health */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.6 }}
            className="lg:col-span-4"
          >
            <SystemHealth />
          </motion.div>

          {/* Trading Positions */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.7 }}
            className="lg:col-span-8"
          >
            <TradingPositions />
          </motion.div>

          {/* Risk Analysis */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.8 }}
            className="lg:col-span-4"
          >
            <RiskAnalysis />
          </motion.div>
        </div>
      </div>
    </div>
  )
}
