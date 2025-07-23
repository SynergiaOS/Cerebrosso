'use client'

import { motion } from 'framer-motion'
import { 
  Globe, 
  Zap, 
  CheckCircle, 
  AlertCircle, 
  XCircle,
  Activity,
  Clock,
  TrendingUp
} from 'lucide-react'

interface Provider {
  status: 'healthy' | 'warning' | 'error'
  usage: number
  responseTime: number
}

interface MultiRpcStatusProps {
  providers: {
    alchemy: Provider
    helius: Provider
    public: Provider
  }
}

const providerConfig = {
  alchemy: {
    name: 'Alchemy',
    description: 'FREE Tier • No RPM Limits',
    limit: '100k/month',
    color: 'blue',
    icon: '🔮'
  },
  helius: {
    name: 'Helius',
    description: 'FREE Tier • Enhanced Data',
    limit: '100k/month',
    color: 'purple',
    icon: '🌟'
  },
  public: {
    name: 'Public RPC',
    description: 'FREE • Unlimited',
    limit: 'Unlimited',
    color: 'emerald',
    icon: '🌐'
  }
}

const statusConfig = {
  healthy: {
    icon: CheckCircle,
    color: 'text-emerald-400',
    bg: 'bg-emerald-500/20',
    border: 'border-emerald-500/30'
  },
  warning: {
    icon: AlertCircle,
    color: 'text-yellow-400',
    bg: 'bg-yellow-500/20',
    border: 'border-yellow-500/30'
  },
  error: {
    icon: XCircle,
    color: 'text-red-400',
    bg: 'bg-red-500/20',
    border: 'border-red-500/30'
  }
}

export function MultiRpcStatus({ providers }: MultiRpcStatusProps) {
  const totalUsage = Object.values(providers).reduce((sum, provider) => sum + provider.usage, 0)
  const avgResponseTime = Math.round(
    Object.values(providers).reduce((sum, provider) => sum + provider.responseTime, 0) / 
    Object.values(providers).length
  )
  const healthyProviders = Object.values(providers).filter(p => p.status === 'healthy').length

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="bg-slate-900/50 backdrop-blur-sm border border-slate-800 rounded-xl p-6"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-3">
          <div className="p-2 rounded-lg bg-gradient-to-br from-blue-500/20 to-purple-500/20 border border-blue-500/30">
            <Globe className="w-5 h-5 text-blue-400" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-white">Multi-RPC Status</h2>
            <p className="text-sm text-slate-400">FREE Providers Only • Real-time Monitoring</p>
          </div>
        </div>

        {/* Summary Stats */}
        <div className="flex items-center space-x-6">
          <div className="text-center">
            <div className="text-2xl font-bold text-emerald-400">{healthyProviders}/3</div>
            <div className="text-xs text-slate-500">Healthy</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-blue-400">{avgResponseTime}ms</div>
            <div className="text-xs text-slate-500">Avg Response</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-purple-400">{totalUsage}%</div>
            <div className="text-xs text-slate-500">Total Usage</div>
          </div>
        </div>
      </div>

      {/* Provider Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {Object.entries(providers).map(([key, provider]) => {
          const config = providerConfig[key as keyof typeof providerConfig]
          const status = statusConfig[provider.status]
          const StatusIcon = status.icon

          return (
            <motion.div
              key={key}
              whileHover={{ scale: 1.02 }}
              className={`
                relative overflow-hidden rounded-lg border ${status.border}
                bg-gradient-to-br from-slate-800/50 to-slate-900/50 p-4
                transition-all duration-300 hover:shadow-lg
              `}
            >
              {/* Provider Header */}
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center space-x-2">
                  <span className="text-lg">{config.icon}</span>
                  <div>
                    <h3 className="font-semibold text-white">{config.name}</h3>
                    <p className="text-xs text-slate-400">{config.description}</p>
                  </div>
                </div>
                <div className={`p-1 rounded-full ${status.bg}`}>
                  <StatusIcon className={`w-4 h-4 ${status.color}`} />
                </div>
              </div>

              {/* Metrics */}
              <div className="space-y-3">
                {/* Usage Bar */}
                <div>
                  <div className="flex justify-between text-xs mb-1">
                    <span className="text-slate-400">Usage</span>
                    <span className="text-white font-medium">{provider.usage}%</span>
                  </div>
                  <div className="w-full bg-slate-700 rounded-full h-2">
                    <motion.div
                      initial={{ width: 0 }}
                      animate={{ width: `${provider.usage}%` }}
                      transition={{ duration: 1, delay: 0.2 }}
                      className={`h-2 rounded-full bg-gradient-to-r ${
                        provider.usage > 80 ? 'from-red-500 to-red-600' :
                        provider.usage > 60 ? 'from-yellow-500 to-yellow-600' :
                        'from-emerald-500 to-emerald-600'
                      }`}
                    />
                  </div>
                </div>

                {/* Response Time */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-1">
                    <Clock className="w-3 h-3 text-slate-400" />
                    <span className="text-xs text-slate-400">Response</span>
                  </div>
                  <span className="text-sm font-medium text-white">
                    {provider.responseTime}ms
                  </span>
                </div>

                {/* Limit */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-1">
                    <Activity className="w-3 h-3 text-slate-400" />
                    <span className="text-xs text-slate-400">Limit</span>
                  </div>
                  <span className="text-sm font-medium text-white">
                    {config.limit}
                  </span>
                </div>
              </div>

              {/* Status Indicator */}
              <div className="absolute top-2 right-2">
                <motion.div
                  animate={{ 
                    scale: provider.status === 'healthy' ? [1, 1.2, 1] : 1,
                    opacity: provider.status === 'healthy' ? [1, 0.7, 1] : 1
                  }}
                  transition={{ 
                    duration: 2, 
                    repeat: provider.status === 'healthy' ? Infinity : 0 
                  }}
                  className={`w-2 h-2 rounded-full ${
                    provider.status === 'healthy' ? 'bg-emerald-400' :
                    provider.status === 'warning' ? 'bg-yellow-400' :
                    'bg-red-400'
                  }`}
                />
              </div>
            </motion.div>
          )
        })}
      </div>

      {/* Footer Stats */}
      <div className="mt-6 pt-4 border-t border-slate-800">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-center">
          <div>
            <div className="text-lg font-bold text-emerald-400">$0</div>
            <div className="text-xs text-slate-500">Monthly Cost</div>
          </div>
          <div>
            <div className="text-lg font-bold text-blue-400">200k+</div>
            <div className="text-xs text-slate-500">Free Requests</div>
          </div>
          <div>
            <div className="text-lg font-bold text-purple-400">95%</div>
            <div className="text-xs text-slate-500">Cost Reduction</div>
          </div>
          <div>
            <div className="text-lg font-bold text-yellow-400">99.9%</div>
            <div className="text-xs text-slate-500">Uptime</div>
          </div>
        </div>
      </div>
    </motion.div>
  )
}
