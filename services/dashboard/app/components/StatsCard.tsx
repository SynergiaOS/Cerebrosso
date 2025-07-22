'use client'

import { motion } from 'framer-motion'
import { ArrowUpIcon, ArrowDownIcon } from '@heroicons/react/24/outline'
import { clsx } from 'clsx'

interface StatsCardProps {
  title: string
  value: string
  icon: React.ComponentType<{ className?: string }>
  trend?: string
  color?: 'primary' | 'success' | 'warning' | 'danger'
  className?: string
}

const colorVariants = {
  primary: {
    icon: 'text-primary-400',
    trend: 'text-primary-400',
    glow: 'shadow-primary-500/20',
  },
  success: {
    icon: 'text-success-400',
    trend: 'text-success-400',
    glow: 'shadow-success-500/20',
  },
  warning: {
    icon: 'text-warning-400',
    trend: 'text-warning-400',
    glow: 'shadow-warning-500/20',
  },
  danger: {
    icon: 'text-danger-400',
    trend: 'text-danger-400',
    glow: 'shadow-danger-500/20',
  },
}

export function StatsCard({
  title,
  value,
  icon: Icon,
  trend,
  color = 'primary',
  className,
}: StatsCardProps) {
  const isPositiveTrend = trend?.startsWith('+')
  const isNegativeTrend = trend?.startsWith('-')
  
  const colors = colorVariants[color]

  return (
    <motion.div
      whileHover={{ scale: 1.02, y: -2 }}
      whileTap={{ scale: 0.98 }}
      className={clsx(
        'glass-card p-6 hover:bg-white/10 transition-all duration-300',
        colors.glow,
        className
      )}
    >
      <div className="flex items-center justify-between">
        <div className="flex-1">
          <p className="text-sm font-medium text-dark-400 uppercase tracking-wide">
            {title}
          </p>
          <p className="text-2xl font-bold text-white mt-1">
            {value}
          </p>
          {trend && (
            <div className="flex items-center mt-2">
              {isPositiveTrend && (
                <ArrowUpIcon className="w-4 h-4 text-success-400 mr-1" />
              )}
              {isNegativeTrend && (
                <ArrowDownIcon className="w-4 h-4 text-danger-400 mr-1" />
              )}
              <span
                className={clsx(
                  'text-sm font-medium',
                  isPositiveTrend && 'text-success-400',
                  isNegativeTrend && 'text-danger-400',
                  !isPositiveTrend && !isNegativeTrend && colors.trend
                )}
              >
                {trend}
              </span>
            </div>
          )}
        </div>
        <div className={clsx('p-3 rounded-lg bg-white/5', colors.icon)}>
          <Icon className="w-6 h-6" />
        </div>
      </div>
    </motion.div>
  )
}
