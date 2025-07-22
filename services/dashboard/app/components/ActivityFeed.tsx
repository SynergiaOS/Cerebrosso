'use client'

import { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { 
  BoltIcon, 
  CheckCircleIcon, 
  ExclamationTriangleIcon,
  CurrencyDollarIcon,
  ClockIcon,
  ArrowTrendingUpIcon
} from '@heroicons/react/24/outline'
import { clsx } from 'clsx'

interface Activity {
  id: string
  type: 'trade' | 'signal' | 'optimization' | 'alert'
  title: string
  description: string
  timestamp: Date
  status: 'success' | 'warning' | 'info'
  value?: number
  token?: string
}

const generateMockActivity = (): Activity => {
  const types = ['trade', 'signal', 'optimization', 'alert'] as const
  const type = types[Math.floor(Math.random() * types.length)]
  
  const activities = {
    trade: {
      title: 'Sandwich Trade Executed',
      description: 'Successfully executed sandwich attack on BONK/SOL pool',
      status: 'success' as const,
      value: 0.0025 + Math.random() * 0.005,
      token: 'BONK'
    },
    signal: {
      title: 'MEV Opportunity Detected',
      description: 'High confidence arbitrage opportunity found',
      status: 'info' as const,
      value: 0.003 + Math.random() * 0.007,
      token: 'RAY'
    },
    optimization: {
      title: 'Strategy Optimized',
      description: 'AI improved sandwich strategy parameters',
      status: 'success' as const
    },
    alert: {
      title: 'High Slippage Warning',
      description: 'Unusual market conditions detected',
      status: 'warning' as const
    }
  }

  return {
    id: Math.random().toString(36).substr(2, 9),
    type,
    ...activities[type],
    timestamp: new Date()
  }
}

const ActivityIcon = ({ type, status }: { type: Activity['type'], status: Activity['status'] }) => {
  const iconClass = clsx(
    'w-5 h-5',
    status === 'success' && 'text-success-400',
    status === 'warning' && 'text-warning-400',
    status === 'info' && 'text-primary-400'
  )

  switch (type) {
    case 'trade':
      return <CurrencyDollarIcon className={iconClass} />
    case 'signal':
      return <BoltIcon className={iconClass} />
    case 'optimization':
      return <ArrowTrendingUpIcon className={iconClass} />
    case 'alert':
      return <ExclamationTriangleIcon className={iconClass} />
    default:
      return <CheckCircleIcon className={iconClass} />
  }
}

export function ActivityFeed() {
  const [activities, setActivities] = useState<Activity[]>([])

  useEffect(() => {
    // Initialize with some activities
    const initialActivities = Array.from({ length: 8 }, generateMockActivity)
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
    
    setActivities(initialActivities)

    // Add new activities periodically
    const interval = setInterval(() => {
      const newActivity = generateMockActivity()
      setActivities(prev => [newActivity, ...prev.slice(0, 19)]) // Keep only last 20
    }, 8000 + Math.random() * 7000) // Random interval between 8-15 seconds

    return () => clearInterval(interval)
  }, [])

  const formatTimeAgo = (timestamp: Date) => {
    const now = new Date()
    const diffInSeconds = Math.floor((now.getTime() - timestamp.getTime()) / 1000)
    
    if (diffInSeconds < 60) return `${diffInSeconds}s ago`
    if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)}m ago`
    return `${Math.floor(diffInSeconds / 3600)}h ago`
  }

  return (
    <div className="glass-card p-6">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-xl font-semibold text-white">
          ⚡ Activity Feed
        </h3>
        <div className="flex items-center space-x-2 text-dark-400">
          <ClockIcon className="w-4 h-4" />
          <span className="text-sm">Real-time</span>
        </div>
      </div>

      <div className="space-y-3 max-h-96 overflow-y-auto custom-scrollbar">
        <AnimatePresence>
          {activities.map((activity, index) => (
            <motion.div
              key={activity.id}
              initial={{ opacity: 0, x: 20, scale: 0.95 }}
              animate={{ opacity: 1, x: 0, scale: 1 }}
              exit={{ opacity: 0, x: -20, scale: 0.95 }}
              transition={{ duration: 0.3, delay: index * 0.05 }}
              className={clsx(
                'p-4 rounded-lg border transition-all duration-200 hover:bg-white/5',
                activity.status === 'success' && 'bg-success-500/5 border-success-500/20',
                activity.status === 'warning' && 'bg-warning-500/5 border-warning-500/20',
                activity.status === 'info' && 'bg-primary-500/5 border-primary-500/20'
              )}
            >
              <div className="flex items-start space-x-3">
                <div className={clsx(
                  'p-2 rounded-lg',
                  activity.status === 'success' && 'bg-success-500/20',
                  activity.status === 'warning' && 'bg-warning-500/20',
                  activity.status === 'info' && 'bg-primary-500/20'
                )}>
                  <ActivityIcon type={activity.type} status={activity.status} />
                </div>
                
                <div className="flex-1 min-w-0">
                  <div className="flex items-center justify-between">
                    <p className="text-sm font-medium text-white truncate">
                      {activity.title}
                    </p>
                    <span className="text-xs text-dark-400 ml-2 flex-shrink-0">
                      {formatTimeAgo(activity.timestamp)}
                    </span>
                  </div>
                  
                  <p className="text-xs text-dark-400 mt-1">
                    {activity.description}
                  </p>
                  
                  {activity.value && (
                    <div className="flex items-center justify-between mt-2">
                      <span className="text-xs text-dark-500">
                        {activity.token && `${activity.token} • `}
                      </span>
                      <span className={clsx(
                        'text-sm font-medium',
                        activity.value > 0 ? 'text-success-400' : 'text-danger-400'
                      )}>
                        {activity.value > 0 ? '+' : ''}{activity.value.toFixed(4)} SOL
                      </span>
                    </div>
                  )}
                </div>
              </div>
            </motion.div>
          ))}
        </AnimatePresence>
      </div>

      {/* Summary Stats */}
      <div className="border-t border-dark-700 pt-4 mt-6">
        <div className="grid grid-cols-3 gap-4 text-center">
          <div>
            <p className="text-lg font-bold text-success-400">
              {activities.filter(a => a.type === 'trade').length}
            </p>
            <p className="text-xs text-dark-400">Trades</p>
          </div>
          <div>
            <p className="text-lg font-bold text-primary-400">
              {activities.filter(a => a.type === 'signal').length}
            </p>
            <p className="text-xs text-dark-400">Signals</p>
          </div>
          <div>
            <p className="text-lg font-bold text-warning-400">
              {activities.filter(a => a.status === 'warning').length}
            </p>
            <p className="text-xs text-dark-400">Alerts</p>
          </div>
        </div>
      </div>
    </div>
  )
}
