'use client'

import { motion } from 'framer-motion'
import { TrendingUp, TrendingDown, DollarSign, Target } from 'lucide-react'

export function TradingPositions() {
  const positions = [
    { 
      id: 1, 
      token: 'SOL/USDC', 
      type: 'Sandwich', 
      profit: 0.045, 
      status: 'completed',
      time: '2m ago'
    },
    { 
      id: 2, 
      token: 'RAY/SOL', 
      type: 'Arbitrage', 
      profit: 0.023, 
      status: 'completed',
      time: '5m ago'
    },
    { 
      id: 3, 
      token: 'ORCA/USDC', 
      type: 'Sandwich', 
      profit: -0.012, 
      status: 'failed',
      time: '8m ago'
    },
    { 
      id: 4, 
      token: 'MNGO/SOL', 
      type: 'MEV', 
      profit: 0.067, 
      status: 'completed',
      time: '12m ago'
    },
  ]

  const totalProfit = positions.reduce((sum, pos) => sum + pos.profit, 0)
  const successRate = (positions.filter(p => p.status === 'completed').length / positions.length) * 100

  return (
    <div className="bg-slate-900/50 backdrop-blur-sm rounded-xl border border-slate-800 p-6">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-lg font-semibold text-white flex items-center">
          <Target className="w-5 h-5 mr-2 text-purple-500" />
          Trading Positions
        </h3>
        <div className="flex items-center space-x-4 text-sm">
          <div className="flex items-center text-emerald-500">
            <DollarSign className="w-4 h-4 mr-1" />
            +{totalProfit.toFixed(3)} SOL
          </div>
          <div className="text-slate-400">
            {successRate.toFixed(0)}% success
          </div>
        </div>
      </div>

      <div className="space-y-3">
        {positions.map((position, index) => (
          <motion.div
            key={position.id}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: index * 0.1 }}
            className="flex items-center justify-between p-4 rounded-lg bg-slate-800/50 border border-slate-700 hover:border-slate-600 transition-colors"
          >
            <div className="flex items-center space-x-4">
              <div className={`w-3 h-3 rounded-full ${
                position.status === 'completed' ? 'bg-emerald-500' : 'bg-red-500'
              }`} />
              
              <div>
                <div className="flex items-center space-x-2">
                  <span className="font-medium text-slate-200">{position.token}</span>
                  <span className="text-xs px-2 py-1 rounded-full bg-slate-700 text-slate-400">
                    {position.type}
                  </span>
                </div>
                <div className="text-xs text-slate-500 mt-1">{position.time}</div>
              </div>
            </div>

            <div className="flex items-center space-x-3">
              <div className={`flex items-center space-x-1 ${
                position.profit > 0 ? 'text-emerald-500' : 'text-red-500'
              }`}>
                {position.profit > 0 ? (
                  <TrendingUp className="w-4 h-4" />
                ) : (
                  <TrendingDown className="w-4 h-4" />
                )}
                <span className="font-medium">
                  {position.profit > 0 ? '+' : ''}{position.profit.toFixed(3)} SOL
                </span>
              </div>
            </div>
          </motion.div>
        ))}
      </div>

      {/* Summary */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.5 }}
        className="mt-6 grid grid-cols-2 gap-4"
      >
        <div className="p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
          <div className="text-sm text-emerald-400 font-medium">Total Profit</div>
          <div className="text-lg font-bold text-emerald-300">
            +{totalProfit.toFixed(3)} SOL
          </div>
        </div>
        <div className="p-3 rounded-lg bg-blue-500/10 border border-blue-500/20">
          <div className="text-sm text-blue-400 font-medium">Success Rate</div>
          <div className="text-lg font-bold text-blue-300">
            {successRate.toFixed(0)}%
          </div>
        </div>
      </motion.div>
    </div>
  )
}
