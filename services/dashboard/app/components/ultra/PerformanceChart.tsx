'use client'

import { motion } from 'framer-motion'
import { BarChart3, TrendingUp } from 'lucide-react'

export function PerformanceChart() {
  const data = [
    { time: '00:00', latency: 120, throughput: 85 },
    { time: '04:00', latency: 110, throughput: 92 },
    { time: '08:00', latency: 95, throughput: 98 },
    { time: '12:00', latency: 127, throughput: 100 },
    { time: '16:00', latency: 105, throughput: 95 },
    { time: '20:00', latency: 115, throughput: 88 },
  ]

  return (
    <div className="bg-slate-900/50 backdrop-blur-sm rounded-xl border border-slate-800 p-6">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-lg font-semibold text-white flex items-center">
          <BarChart3 className="w-5 h-5 mr-2 text-blue-500" />
          Performance Chart
        </h3>
        <div className="flex items-center text-sm text-emerald-500">
          <TrendingUp className="w-4 h-4 mr-1" />
          +12% today
        </div>
      </div>

      <div className="space-y-4">
        {/* Simple bar chart visualization */}
        <div className="grid grid-cols-6 gap-2 h-32">
          {data.map((point, index) => (
            <motion.div
              key={point.time}
              initial={{ height: 0 }}
              animate={{ height: `${(point.throughput / 100) * 100}%` }}
              transition={{ delay: index * 0.1, duration: 0.5 }}
              className="bg-gradient-to-t from-blue-500 to-emerald-500 rounded-t-sm flex flex-col justify-end"
            >
              <div className="text-xs text-white text-center p-1">
                {point.throughput}%
              </div>
            </motion.div>
          ))}
        </div>

        {/* Time labels */}
        <div className="grid grid-cols-6 gap-2 text-xs text-slate-400 text-center">
          {data.map((point) => (
            <div key={point.time}>{point.time}</div>
          ))}
        </div>

        {/* Legend */}
        <div className="flex items-center justify-center space-x-6 pt-4 border-t border-slate-700">
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 bg-blue-500 rounded-full" />
            <span className="text-sm text-slate-400">Throughput</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 bg-emerald-500 rounded-full" />
            <span className="text-sm text-slate-400">Latency</span>
          </div>
        </div>
      </div>
    </div>
  )
}
