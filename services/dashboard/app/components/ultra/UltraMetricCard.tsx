'use client'

import { motion } from 'framer-motion'
import { LucideIcon, ArrowUp, ArrowDown, Minus } from 'lucide-react'
import { LineChart, Line, ResponsiveContainer } from 'recharts'

interface UltraMetricCardProps {
  title: string
  value: string
  subtitle?: string
  change: string
  icon: LucideIcon
  color: 'emerald' | 'blue' | 'yellow' | 'red' | 'purple'
  trend: 'up' | 'down' | 'neutral'
  sparkline?: number[]
}

const colorClasses = {
  emerald: {
    bg: 'from-emerald-500/20 to-emerald-600/20',
    border: 'border-emerald-500/30',
    icon: 'text-emerald-400',
    text: 'text-emerald-400',
    sparkline: '#10b981'
  },
  blue: {
    bg: 'from-blue-500/20 to-blue-600/20',
    border: 'border-blue-500/30',
    icon: 'text-blue-400',
    text: 'text-blue-400',
    sparkline: '#3b82f6'
  },
  yellow: {
    bg: 'from-yellow-500/20 to-yellow-600/20',
    border: 'border-yellow-500/30',
    icon: 'text-yellow-400',
    text: 'text-yellow-400',
    sparkline: '#eab308'
  },
  red: {
    bg: 'from-red-500/20 to-red-600/20',
    border: 'border-red-500/30',
    icon: 'text-red-400',
    text: 'text-red-400',
    sparkline: '#ef4444'
  },
  purple: {
    bg: 'from-purple-500/20 to-purple-600/20',
    border: 'border-purple-500/30',
    icon: 'text-purple-400',
    text: 'text-purple-400',
    sparkline: '#a855f7'
  }
}

const trendIcons = {
  up: ArrowUp,
  down: ArrowDown,
  neutral: Minus
}

const trendColors = {
  up: 'text-emerald-400',
  down: 'text-red-400',
  neutral: 'text-slate-400'
}

export function UltraMetricCard({
  title,
  value,
  subtitle,
  change,
  icon: Icon,
  color,
  trend,
  sparkline = []
}: UltraMetricCardProps) {
  const colors = colorClasses[color]
  const TrendIcon = trendIcons[trend]
  const trendColor = trendColors[trend]

  // Convert sparkline data for recharts
  const chartData = sparkline.map((value, index) => ({ value, index }))

  return (
    <motion.div
      whileHover={{ scale: 1.02, y: -2 }}
      whileTap={{ scale: 0.98 }}
      className={`
        relative overflow-hidden rounded-xl border ${colors.border}
        bg-gradient-to-br ${colors.bg} backdrop-blur-sm
        p-4 transition-all duration-300 hover:shadow-lg hover:shadow-${color}-500/10
      `}
    >
      {/* Background Pattern */}
      <div className="absolute inset-0 opacity-5">
        <div className="absolute inset-0 bg-gradient-to-br from-white/10 to-transparent" />
        <div className="absolute top-0 right-0 w-32 h-32 bg-gradient-radial from-white/5 to-transparent" />
      </div>

      {/* Content */}
      <div className="relative z-10">
        {/* Header */}
        <div className="flex items-center justify-between mb-3">
          <div className={`p-2 rounded-lg bg-gradient-to-br ${colors.bg} border ${colors.border}`}>
            <Icon className={`w-4 h-4 ${colors.icon}`} />
          </div>
          
          {sparkline.length > 0 && (
            <div className="w-16 h-8">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={chartData}>
                  <Line
                    type="monotone"
                    dataKey="value"
                    stroke={colors.sparkline}
                    strokeWidth={2}
                    dot={false}
                    activeDot={false}
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>
          )}
        </div>

        {/* Title */}
        <h3 className="text-sm font-medium text-slate-400 mb-1">
          {title}
        </h3>

        {/* Value */}
        <div className="flex items-baseline space-x-1 mb-2">
          <span className="text-2xl font-bold text-white">
            {value}
          </span>
          {subtitle && (
            <span className="text-sm text-slate-500">
              {subtitle}
            </span>
          )}
        </div>

        {/* Change */}
        <div className="flex items-center space-x-1">
          <TrendIcon className={`w-3 h-3 ${trendColor}`} />
          <span className={`text-xs font-medium ${trendColor}`}>
            {change}
          </span>
        </div>
      </div>

      {/* Hover Effect */}
      <motion.div
        className={`absolute inset-0 bg-gradient-to-r ${colors.bg} opacity-0 transition-opacity duration-300`}
        whileHover={{ opacity: 0.1 }}
      />
    </motion.div>
  )
}
