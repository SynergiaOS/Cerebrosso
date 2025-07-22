'use client'

import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { 
  ChartBarIcon, 
  CpuChipIcon, 
  BoltIcon, 
  EyeIcon,
  CurrencyDollarIcon,
  ClockIcon
} from '@heroicons/react/24/outline'
import { StatsCard } from './components/StatsCard'
import { TradingChart } from './components/TradingChart'
import { ActivityFeed } from './components/ActivityFeed'
import { SystemStatus } from './components/SystemStatus'
import { PerformanceMetrics } from './components/PerformanceMetrics'

export default function Dashboard() {
  const [isLoading, setIsLoading] = useState(true)
  const [systemData, setSystemData] = useState({
    dailyROI: 4.7,
    totalTrades: 127,
    successRate: 85.2,
    avgExecutionTime: 87,
    activeProfitSOL: 0.34,
    systemUptime: 99.9
  })

  useEffect(() => {
    // Simulate loading
    const timer = setTimeout(() => setIsLoading(false), 1000)
    return () => clearTimeout(timer)
  }, [])

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <motion.div
          animate={{ rotate: 360 }}
          transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
          className="w-16 h-16 border-4 border-primary-500 border-t-transparent rounded-full"
        />
      </div>
    )
  }

  return (
    <div className="min-h-screen p-6">
      {/* Header */}
      <motion.header
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="mb-8"
      >
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-4xl font-bold bg-gradient-to-r from-primary-400 to-secondary-400 bg-clip-text text-transparent">
              🐺 Cerberus Phoenix v2.0
            </h1>
            <p className="text-dark-400 mt-2">
              Autonomiczny ekosystem do operacji on-chain na Solanie
            </p>
          </div>
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2 text-success-400">
              <div className="w-3 h-3 bg-success-400 rounded-full animate-pulse" />
              <span className="text-sm font-medium">System Online</span>
            </div>
          </div>
        </div>
      </motion.header>

      {/* Stats Grid */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-6 mb-8"
      >
        <StatsCard
          title="Daily ROI"
          value={`${systemData.dailyROI}%`}
          icon={ChartBarIcon}
          trend="+0.3%"
          color="success"
        />
        <StatsCard
          title="Total Trades"
          value={systemData.totalTrades.toString()}
          icon={CurrencyDollarIcon}
          trend="+12"
          color="primary"
        />
        <StatsCard
          title="Success Rate"
          value={`${systemData.successRate}%`}
          icon={EyeIcon}
          trend="+2.1%"
          color="success"
        />
        <StatsCard
          title="Avg Execution"
          value={`${systemData.avgExecutionTime}ms`}
          icon={BoltIcon}
          trend="-5ms"
          color="warning"
        />
        <StatsCard
          title="Profit (SOL)"
          value={systemData.activeProfitSOL.toFixed(3)}
          icon={CpuChipIcon}
          trend="+0.025"
          color="success"
        />
        <StatsCard
          title="Uptime"
          value={`${systemData.systemUptime}%`}
          icon={ClockIcon}
          trend="99.9%"
          color="success"
        />
      </motion.div>

      {/* Main Content Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Trading Chart */}
        <motion.div
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.2 }}
          className="lg:col-span-2"
        >
          <TradingChart />
        </motion.div>

        {/* System Status */}
        <motion.div
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.3 }}
        >
          <SystemStatus />
        </motion.div>

        {/* Performance Metrics */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
          className="lg:col-span-2"
        >
          <PerformanceMetrics />
        </motion.div>

        {/* Activity Feed */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.5 }}
        >
          <ActivityFeed />
        </motion.div>
      </div>
    </div>
  )
}
