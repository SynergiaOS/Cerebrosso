'use client'

import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { 
  BarChart, 
  Bar, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell
} from 'recharts'

// 📊 Mock performance data
const generatePerformanceData = () => {
  const strategies = ['Sandwich', 'Arbitrage', 'Piranha Surf', 'MEV Snipe']
  
  return strategies.map(strategy => ({
    strategy,
    trades: Math.floor(Math.random() * 50) + 20,
    profit: Number((Math.random() * 0.1 + 0.02).toFixed(4)),
    successRate: Number((75 + Math.random() * 20).toFixed(1)),
    avgExecutionTime: Math.floor(Math.random() * 50) + 60
  }))
}

const generateTimeData = () => {
  const hours = []
  for (let i = 23; i >= 0; i--) {
    const hour = new Date(Date.now() - i * 60 * 60 * 1000).getHours()
    hours.push({
      hour: `${hour.toString().padStart(2, '0')}:00`,
      trades: Math.floor(Math.random() * 20) + 5,
      profit: Number((Math.random() * 0.05 + 0.01).toFixed(4)),
      gasUsed: Math.floor(Math.random() * 10000) + 5000
    })
  }
  return hours
}

const COLORS = ['#22c55e', '#3b82f6', '#f59e0b', '#ef4444', '#8b5cf6']

export function PerformanceMetrics() {
  const [strategyData, setStrategyData] = useState(generatePerformanceData())
  const [timeData, setTimeData] = useState(generateTimeData())
  const [activeView, setActiveView] = useState<'strategies' | 'timeline'>('strategies')

  useEffect(() => {
    const interval = setInterval(() => {
      setStrategyData(generatePerformanceData())
      setTimeData(generateTimeData())
    }, 30000) // Update every 30 seconds

    return () => clearInterval(interval)
  }, [])

  const totalTrades = strategyData.reduce((sum, item) => sum + item.trades, 0)
  const totalProfit = strategyData.reduce((sum, item) => sum + item.profit, 0)
  const avgSuccessRate = strategyData.reduce((sum, item) => sum + item.successRate, 0) / strategyData.length

  return (
    <div className="glass-card p-6">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-xl font-semibold text-white">
          📊 Performance Metrics
        </h3>
        
        {/* View Toggle */}
        <div className="flex space-x-1 bg-dark-800 rounded-lg p-1">
          <button
            onClick={() => setActiveView('strategies')}
            className={`px-3 py-1.5 text-sm font-medium rounded-md transition-all duration-200 ${
              activeView === 'strategies'
                ? 'bg-white/10 text-white'
                : 'text-dark-400 hover:text-white'
            }`}
          >
            Strategies
          </button>
          <button
            onClick={() => setActiveView('timeline')}
            className={`px-3 py-1.5 text-sm font-medium rounded-md transition-all duration-200 ${
              activeView === 'timeline'
                ? 'bg-white/10 text-white'
                : 'text-dark-400 hover:text-white'
            }`}
          >
            Timeline
          </button>
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-3 gap-4 mb-6">
        <div className="bg-dark-800/50 rounded-lg p-4 text-center">
          <p className="text-2xl font-bold text-primary-400">{totalTrades}</p>
          <p className="text-sm text-dark-400">Total Trades</p>
        </div>
        <div className="bg-dark-800/50 rounded-lg p-4 text-center">
          <p className="text-2xl font-bold text-success-400">
            {totalProfit.toFixed(4)}
          </p>
          <p className="text-sm text-dark-400">Total Profit (SOL)</p>
        </div>
        <div className="bg-dark-800/50 rounded-lg p-4 text-center">
          <p className="text-2xl font-bold text-warning-400">
            {avgSuccessRate.toFixed(1)}%
          </p>
          <p className="text-sm text-dark-400">Avg Success Rate</p>
        </div>
      </div>

      {/* Charts */}
      <motion.div
        key={activeView}
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.3 }}
        className="h-80"
      >
        {activeView === 'strategies' ? (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 h-full">
            {/* Strategy Performance Bar Chart */}
            <div>
              <h4 className="text-sm font-medium text-white mb-4">
                Strategy Performance
              </h4>
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={strategyData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                  <XAxis 
                    dataKey="strategy" 
                    stroke="#9ca3af"
                    fontSize={12}
                    angle={-45}
                    textAnchor="end"
                    height={60}
                  />
                  <YAxis 
                    stroke="#9ca3af"
                    fontSize={12}
                  />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: '#1e293b',
                      border: '1px solid #334155',
                      borderRadius: '0.5rem',
                      color: '#f8fafc'
                    }}
                  />
                  <Bar 
                    dataKey="profit" 
                    fill="#22c55e"
                    radius={[4, 4, 0, 0]}
                  />
                </BarChart>
              </ResponsiveContainer>
            </div>

            {/* Strategy Distribution Pie Chart */}
            <div>
              <h4 className="text-sm font-medium text-white mb-4">
                Trade Distribution
              </h4>
              <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                  <Pie
                    data={strategyData}
                    cx="50%"
                    cy="50%"
                    outerRadius={80}
                    dataKey="trades"
                    label={({ strategy, percent }) => 
                      `${strategy} ${(percent * 100).toFixed(0)}%`
                    }
                    labelLine={false}
                  >
                    {strategyData.map((entry, index) => (
                      <Cell 
                        key={`cell-${index}`} 
                        fill={COLORS[index % COLORS.length]} 
                      />
                    ))}
                  </Pie>
                  <Tooltip
                    contentStyle={{
                      backgroundColor: '#1e293b',
                      border: '1px solid #334155',
                      borderRadius: '0.5rem',
                      color: '#f8fafc'
                    }}
                  />
                </PieChart>
              </ResponsiveContainer>
            </div>
          </div>
        ) : (
          <div>
            <h4 className="text-sm font-medium text-white mb-4">
              24-Hour Trading Timeline
            </h4>
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={timeData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis 
                  dataKey="hour" 
                  stroke="#9ca3af"
                  fontSize={12}
                />
                <YAxis 
                  stroke="#9ca3af"
                  fontSize={12}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: '#1e293b',
                    border: '1px solid #334155',
                    borderRadius: '0.5rem',
                    color: '#f8fafc'
                  }}
                />
                <Bar 
                  dataKey="trades" 
                  fill="#3b82f6"
                  radius={[2, 2, 0, 0]}
                />
              </BarChart>
            </ResponsiveContainer>
          </div>
        )}
      </motion.div>

      {/* Strategy Details Table */}
      {activeView === 'strategies' && (
        <div className="mt-6 border-t border-dark-700 pt-6">
          <h4 className="text-sm font-medium text-white mb-4">
            Strategy Details
          </h4>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-dark-400 border-b border-dark-700">
                  <th className="text-left py-2">Strategy</th>
                  <th className="text-right py-2">Trades</th>
                  <th className="text-right py-2">Profit (SOL)</th>
                  <th className="text-right py-2">Success Rate</th>
                  <th className="text-right py-2">Avg Time (ms)</th>
                </tr>
              </thead>
              <tbody>
                {strategyData.map((strategy, index) => (
                  <tr key={strategy.strategy} className="border-b border-dark-800">
                    <td className="py-3 text-white font-medium">
                      {strategy.strategy}
                    </td>
                    <td className="py-3 text-right text-primary-400">
                      {strategy.trades}
                    </td>
                    <td className="py-3 text-right text-success-400">
                      {strategy.profit.toFixed(4)}
                    </td>
                    <td className="py-3 text-right text-warning-400">
                      {strategy.successRate}%
                    </td>
                    <td className="py-3 text-right text-dark-300">
                      {strategy.avgExecutionTime}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  )
}
