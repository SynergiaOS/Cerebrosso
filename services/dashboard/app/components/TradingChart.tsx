'use client'

import { useState, useEffect } from 'react'
import { 
  LineChart, 
  Line, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  Area,
  AreaChart
} from 'recharts'
import { motion } from 'framer-motion'

// 📊 Mock data generator
const generateMockData = () => {
  const data = []
  const now = new Date()
  
  for (let i = 23; i >= 0; i--) {
    const time = new Date(now.getTime() - i * 60 * 60 * 1000)
    const baseProfit = 0.3 + Math.random() * 0.4
    
    data.push({
      time: time.toLocaleTimeString('en-US', { 
        hour: '2-digit', 
        minute: '2-digit' 
      }),
      profit: Number(baseProfit.toFixed(4)),
      trades: Math.floor(Math.random() * 15) + 5,
      successRate: 75 + Math.random() * 20,
    })
  }
  
  return data
}

export function TradingChart() {
  const [data, setData] = useState(generateMockData())
  const [activeTab, setActiveTab] = useState<'profit' | 'trades' | 'success'>('profit')

  useEffect(() => {
    const interval = setInterval(() => {
      setData(generateMockData())
    }, 30000) // Update every 30 seconds

    return () => clearInterval(interval)
  }, [])

  const tabs = [
    { id: 'profit', label: 'Profit (SOL)', color: '#22c55e' },
    { id: 'trades', label: 'Trades', color: '#3b82f6' },
    { id: 'success', label: 'Success Rate', color: '#f59e0b' },
  ]

  const getChartData = () => {
    switch (activeTab) {
      case 'profit':
        return { dataKey: 'profit', color: '#22c55e', suffix: ' SOL' }
      case 'trades':
        return { dataKey: 'trades', color: '#3b82f6', suffix: '' }
      case 'success':
        return { dataKey: 'successRate', color: '#f59e0b', suffix: '%' }
      default:
        return { dataKey: 'profit', color: '#22c55e', suffix: ' SOL' }
    }
  }

  const chartConfig = getChartData()

  return (
    <div className="glass-card p-6">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-xl font-semibold text-white">
          📈 Trading Performance (24h)
        </h3>
        
        {/* Tab Selector */}
        <div className="flex space-x-1 bg-dark-800 rounded-lg p-1">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`px-3 py-1.5 text-sm font-medium rounded-md transition-all duration-200 ${
                activeTab === tab.id
                  ? 'bg-white/10 text-white'
                  : 'text-dark-400 hover:text-white'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      <motion.div
        key={activeTab}
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.3 }}
        className="h-80"
      >
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data}>
            <defs>
              <linearGradient id="colorGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={chartConfig.color} stopOpacity={0.3}/>
                <stop offset="95%" stopColor={chartConfig.color} stopOpacity={0}/>
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
            <XAxis 
              dataKey="time" 
              stroke="#9ca3af"
              fontSize={12}
              tickLine={false}
            />
            <YAxis 
              stroke="#9ca3af"
              fontSize={12}
              tickLine={false}
              tickFormatter={(value) => `${value}${chartConfig.suffix}`}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: '#1e293b',
                border: '1px solid #334155',
                borderRadius: '0.5rem',
                color: '#f8fafc'
              }}
              formatter={(value: any) => [`${value}${chartConfig.suffix}`, tabs.find(t => t.id === activeTab)?.label]}
              labelStyle={{ color: '#94a3b8' }}
            />
            <Area
              type="monotone"
              dataKey={chartConfig.dataKey}
              stroke={chartConfig.color}
              strokeWidth={2}
              fill="url(#colorGradient)"
              dot={{ fill: chartConfig.color, strokeWidth: 2, r: 4 }}
              activeDot={{ r: 6, stroke: chartConfig.color, strokeWidth: 2 }}
            />
          </AreaChart>
        </ResponsiveContainer>
      </motion.div>

      {/* Quick Stats */}
      <div className="grid grid-cols-3 gap-4 mt-6 pt-6 border-t border-dark-700">
        <div className="text-center">
          <p className="text-2xl font-bold text-success-400">
            {data[data.length - 1]?.profit.toFixed(4)}
          </p>
          <p className="text-sm text-dark-400">Current Profit</p>
        </div>
        <div className="text-center">
          <p className="text-2xl font-bold text-primary-400">
            {data.reduce((sum, item) => sum + item.trades, 0)}
          </p>
          <p className="text-sm text-dark-400">Total Trades</p>
        </div>
        <div className="text-center">
          <p className="text-2xl font-bold text-warning-400">
            {(data.reduce((sum, item) => sum + item.successRate, 0) / data.length).toFixed(1)}%
          </p>
          <p className="text-sm text-dark-400">Avg Success</p>
        </div>
      </div>
    </div>
  )
}
