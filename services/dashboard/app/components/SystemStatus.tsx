'use client'

import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { 
  CheckCircleIcon, 
  ExclamationTriangleIcon, 
  XCircleIcon,
  ClockIcon,
  CpuChipIcon,
  ServerIcon
} from '@heroicons/react/24/outline'
import { clsx } from 'clsx'

interface ServiceStatus {
  name: string
  status: 'online' | 'warning' | 'offline'
  uptime: string
  lastCheck: string
  responseTime?: number
}

const mockServices: ServiceStatus[] = [
  {
    name: 'HFT-Ninja',
    status: 'online',
    uptime: '99.9%',
    lastCheck: '2s ago',
    responseTime: 87
  },
  {
    name: 'Cerebro-BFF',
    status: 'online',
    uptime: '99.8%',
    lastCheck: '1s ago',
    responseTime: 124
  },
  {
    name: 'Qdrant Vector DB',
    status: 'online',
    uptime: '100%',
    lastCheck: '3s ago',
    responseTime: 45
  },
  {
    name: 'Kestra Orchestrator',
    status: 'warning',
    uptime: '98.2%',
    lastCheck: '5s ago',
    responseTime: 234
  },
  {
    name: 'Solana RPC',
    status: 'online',
    uptime: '99.5%',
    lastCheck: '1s ago',
    responseTime: 156
  },
  {
    name: 'Jito Block Engine',
    status: 'online',
    uptime: '99.7%',
    lastCheck: '2s ago',
    responseTime: 98
  }
]

const StatusIcon = ({ status }: { status: ServiceStatus['status'] }) => {
  switch (status) {
    case 'online':
      return <CheckCircleIcon className="w-5 h-5 text-success-400" />
    case 'warning':
      return <ExclamationTriangleIcon className="w-5 h-5 text-warning-400" />
    case 'offline':
      return <XCircleIcon className="w-5 h-5 text-danger-400" />
  }
}

const StatusBadge = ({ status }: { status: ServiceStatus['status'] }) => {
  const variants = {
    online: 'bg-success-500/20 text-success-400 border-success-500/30',
    warning: 'bg-warning-500/20 text-warning-400 border-warning-500/30',
    offline: 'bg-danger-500/20 text-danger-400 border-danger-500/30'
  }

  return (
    <span className={clsx(
      'px-2 py-1 text-xs font-medium rounded-full border',
      variants[status]
    )}>
      {status.charAt(0).toUpperCase() + status.slice(1)}
    </span>
  )
}

export function SystemStatus() {
  const [services, setServices] = useState(mockServices)
  const [systemMetrics, setSystemMetrics] = useState({
    cpu: 23.5,
    memory: 67.2,
    disk: 45.8,
    network: 12.3
  })

  useEffect(() => {
    const interval = setInterval(() => {
      // Simulate real-time updates
      setSystemMetrics({
        cpu: 20 + Math.random() * 15,
        memory: 60 + Math.random() * 20,
        disk: 40 + Math.random() * 20,
        network: 10 + Math.random() * 10
      })
    }, 5000)

    return () => clearInterval(interval)
  }, [])

  const onlineServices = services.filter(s => s.status === 'online').length
  const totalServices = services.length

  return (
    <div className="glass-card p-6">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-xl font-semibold text-white">
          🔧 System Status
        </h3>
        <div className="flex items-center space-x-2">
          <div className="w-3 h-3 bg-success-400 rounded-full animate-pulse" />
          <span className="text-sm text-success-400 font-medium">
            {onlineServices}/{totalServices} Online
          </span>
        </div>
      </div>

      {/* Services Status */}
      <div className="space-y-3 mb-6">
        {services.map((service, index) => (
          <motion.div
            key={service.name}
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: index * 0.1 }}
            className="flex items-center justify-between p-3 bg-dark-800/50 rounded-lg hover:bg-dark-800 transition-colors duration-200"
          >
            <div className="flex items-center space-x-3">
              <StatusIcon status={service.status} />
              <div>
                <p className="text-sm font-medium text-white">
                  {service.name}
                </p>
                <p className="text-xs text-dark-400">
                  {service.responseTime}ms • {service.lastCheck}
                </p>
              </div>
            </div>
            <div className="flex items-center space-x-3">
              <span className="text-xs text-dark-400">
                {service.uptime}
              </span>
              <StatusBadge status={service.status} />
            </div>
          </motion.div>
        ))}
      </div>

      {/* System Metrics */}
      <div className="border-t border-dark-700 pt-6">
        <h4 className="text-sm font-medium text-white mb-4 flex items-center">
          <ServerIcon className="w-4 h-4 mr-2" />
          System Metrics
        </h4>
        
        <div className="space-y-3">
          {[
            { label: 'CPU Usage', value: systemMetrics.cpu, color: 'bg-primary-500' },
            { label: 'Memory', value: systemMetrics.memory, color: 'bg-success-500' },
            { label: 'Disk I/O', value: systemMetrics.disk, color: 'bg-warning-500' },
            { label: 'Network', value: systemMetrics.network, color: 'bg-secondary-500' }
          ].map((metric) => (
            <div key={metric.label}>
              <div className="flex justify-between text-sm mb-1">
                <span className="text-dark-400">{metric.label}</span>
                <span className="text-white font-medium">
                  {metric.value.toFixed(1)}%
                </span>
              </div>
              <div className="w-full bg-dark-700 rounded-full h-2">
                <motion.div
                  className={clsx('h-2 rounded-full', metric.color)}
                  initial={{ width: 0 }}
                  animate={{ width: `${metric.value}%` }}
                  transition={{ duration: 0.5, ease: 'easeOut' }}
                />
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Quick Actions */}
      <div className="border-t border-dark-700 pt-6 mt-6">
        <div className="grid grid-cols-2 gap-3">
          <button className="btn-secondary text-xs py-2">
            🔄 Restart Services
          </button>
          <button className="btn-secondary text-xs py-2">
            📊 View Logs
          </button>
        </div>
      </div>
    </div>
  )
}
