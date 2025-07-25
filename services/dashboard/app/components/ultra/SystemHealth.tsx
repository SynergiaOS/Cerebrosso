'use client'

import { motion } from 'framer-motion'
import { Cpu, Database, Wifi, CheckCircle, AlertTriangle } from 'lucide-react'

export function SystemHealth() {
  const healthMetrics = [
    { name: 'CPU Usage', value: 45, status: 'healthy', icon: Cpu },
    { name: 'Memory', value: 62, status: 'healthy', icon: Database },
    { name: 'Network', value: 28, status: 'healthy', icon: Wifi },
  ]

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'healthy': return 'text-emerald-500'
      case 'warning': return 'text-yellow-500'
      case 'critical': return 'text-red-500'
      default: return 'text-slate-500'
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'healthy': return CheckCircle
      case 'warning': return AlertTriangle
      case 'critical': return AlertTriangle
      default: return CheckCircle
    }
  }

  return (
    <div className="bg-slate-900/50 backdrop-blur-sm rounded-xl border border-slate-800 p-6">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-lg font-semibold text-white flex items-center">
          <CheckCircle className="w-5 h-5 mr-2 text-emerald-500" />
          System Health
        </h3>
        <div className="text-sm text-emerald-500 font-medium">
          All Systems Operational
        </div>
      </div>

      <div className="space-y-4">
        {healthMetrics.map((metric, index) => {
          const Icon = metric.icon
          const StatusIcon = getStatusIcon(metric.status)
          
          return (
            <motion.div
              key={metric.name}
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: index * 0.1 }}
              className="flex items-center justify-between p-3 rounded-lg bg-slate-800/50 border border-slate-700"
            >
              <div className="flex items-center space-x-3">
                <Icon className="w-5 h-5 text-slate-400" />
                <span className="text-sm font-medium text-slate-300">{metric.name}</span>
              </div>
              
              <div className="flex items-center space-x-3">
                <div className="flex items-center space-x-2">
                  <span className="text-sm text-slate-400">{metric.value}%</span>
                  <div className="w-16 h-2 bg-slate-700 rounded-full overflow-hidden">
                    <motion.div
                      initial={{ width: 0 }}
                      animate={{ width: `${metric.value}%` }}
                      transition={{ delay: index * 0.1 + 0.2, duration: 0.5 }}
                      className={`h-full rounded-full ${
                        metric.value < 70 ? 'bg-emerald-500' : 
                        metric.value < 85 ? 'bg-yellow-500' : 'bg-red-500'
                      }`}
                    />
                  </div>
                </div>
                <StatusIcon className={`w-4 h-4 ${getStatusColor(metric.status)}`} />
              </div>
            </motion.div>
          )
        })}
      </div>

      {/* Overall Status */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.5 }}
        className="mt-6 p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/20"
      >
        <div className="flex items-center justify-center space-x-2">
          <CheckCircle className="w-5 h-5 text-emerald-500" />
          <span className="text-sm font-medium text-emerald-400">
            System running optimally
          </span>
        </div>
      </motion.div>
    </div>
  )
}
