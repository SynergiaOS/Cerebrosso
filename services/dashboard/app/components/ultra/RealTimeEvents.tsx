'use client'

import { motion } from 'framer-motion'
import { Activity, Clock, TrendingUp } from 'lucide-react'

export function RealTimeEvents() {
  const events = [
    { id: 1, type: 'trade', message: 'Sandwich opportunity detected', time: '2s ago', status: 'success' },
    { id: 2, type: 'rpc', message: 'Switched to Alchemy RPC', time: '5s ago', status: 'info' },
    { id: 3, type: 'profit', message: 'Profit: +0.045 SOL', time: '12s ago', status: 'success' },
    { id: 4, type: 'system', message: 'Health check passed', time: '30s ago', status: 'info' },
  ]

  return (
    <div className="bg-slate-900/50 backdrop-blur-sm rounded-xl border border-slate-800 p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-white flex items-center">
          <Activity className="w-5 h-5 mr-2 text-emerald-500" />
          Real-time Events
        </h3>
        <div className="flex items-center text-sm text-slate-400">
          <Clock className="w-4 h-4 mr-1" />
          Live
        </div>
      </div>

      <div className="space-y-3">
        {events.map((event, index) => (
          <motion.div
            key={event.id}
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: index * 0.1 }}
            className="flex items-center justify-between p-3 rounded-lg bg-slate-800/50 border border-slate-700"
          >
            <div className="flex items-center space-x-3">
              <div className={`w-2 h-2 rounded-full ${
                event.status === 'success' ? 'bg-emerald-500' : 'bg-blue-500'
              }`} />
              <span className="text-sm text-slate-300">{event.message}</span>
            </div>
            <span className="text-xs text-slate-500">{event.time}</span>
          </motion.div>
        ))}
      </div>
    </div>
  )
}
