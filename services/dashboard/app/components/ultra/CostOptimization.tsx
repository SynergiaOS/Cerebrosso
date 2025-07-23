'use client'

import { motion } from 'framer-motion'
import { DollarSign, TrendingDown, Zap, Target } from 'lucide-react'
import { PieChart, Pie, Cell, ResponsiveContainer, BarChart, Bar, XAxis, YAxis, Tooltip } from 'recharts'

interface CostOptimizationProps {
  savings: number
}

const costData = [
  { name: 'Before (Single Provider)', value: 140, color: '#ef4444' },
  { name: 'After (Multi-RPC)', value: 13, color: '#10b981' },
]

const providerCosts = [
  { name: 'Alchemy', cost: 0, requests: 100000, color: '#3b82f6' },
  { name: 'Helius', cost: 0, requests: 100000, color: '#8b5cf6' },
  { name: 'Public RPC', cost: 0, requests: 999999, color: '#10b981' },
]

const monthlyTrend = [
  { month: 'Jan', traditional: 140, optimized: 13 },
  { month: 'Feb', traditional: 145, optimized: 15 },
  { month: 'Mar', traditional: 138, optimized: 12 },
  { month: 'Apr', traditional: 142, optimized: 14 },
  { month: 'May', traditional: 139, optimized: 13 },
  { month: 'Jun', traditional: 144, optimized: 13 },
]

export function CostOptimization({ savings }: CostOptimizationProps) {
  const savingsPercentage = Math.round(((140 - 13) / 140) * 100)
  const annualSavings = savings * 12

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="bg-slate-900/50 backdrop-blur-sm border border-slate-800 rounded-xl p-6"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-3">
          <div className="p-2 rounded-lg bg-gradient-to-br from-emerald-500/20 to-green-500/20 border border-emerald-500/30">
            <DollarSign className="w-5 h-5 text-emerald-400" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-white">Cost Optimization</h2>
            <p className="text-sm text-slate-400">Multi-RPC vs Single Provider</p>
          </div>
        </div>

        {/* Key Metrics */}
        <div className="text-right">
          <div className="text-2xl font-bold text-emerald-400">${savings}/mo</div>
          <div className="text-sm text-slate-400">Saved</div>
        </div>
      </div>

      {/* Cost Comparison */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        {/* Pie Chart */}
        <div className="bg-slate-800/30 rounded-lg p-4">
          <h3 className="text-sm font-medium text-slate-300 mb-4">Monthly Cost Comparison</h3>
          <div className="h-48">
            <ResponsiveContainer width="100%" height="100%">
              <PieChart>
                <Pie
                  data={costData}
                  cx="50%"
                  cy="50%"
                  innerRadius={40}
                  outerRadius={80}
                  paddingAngle={5}
                  dataKey="value"
                >
                  {costData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip 
                  formatter={(value) => [`$${value}`, 'Cost']}
                  contentStyle={{
                    backgroundColor: '#1e293b',
                    border: '1px solid #334155',
                    borderRadius: '8px',
                    color: '#f1f5f9'
                  }}
                />
              </PieChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Savings Breakdown */}
        <div className="space-y-4">
          <h3 className="text-sm font-medium text-slate-300">Savings Breakdown</h3>
          
          {/* Annual Savings */}
          <div className="bg-slate-800/30 rounded-lg p-4">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm text-slate-400">Annual Savings</span>
              <span className="text-lg font-bold text-emerald-400">${annualSavings}</span>
            </div>
            <div className="w-full bg-slate-700 rounded-full h-2">
              <motion.div
                initial={{ width: 0 }}
                animate={{ width: `${savingsPercentage}%` }}
                transition={{ duration: 1.5, delay: 0.5 }}
                className="h-2 rounded-full bg-gradient-to-r from-emerald-500 to-green-500"
              />
            </div>
            <div className="text-xs text-slate-500 mt-1">{savingsPercentage}% reduction</div>
          </div>

          {/* Provider Costs */}
          <div className="space-y-2">
            {providerCosts.map((provider, index) => (
              <motion.div
                key={provider.name}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: index * 0.1 }}
                className="flex items-center justify-between p-3 bg-slate-800/30 rounded-lg"
              >
                <div className="flex items-center space-x-3">
                  <div 
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: provider.color }}
                  />
                  <span className="text-sm text-white">{provider.name}</span>
                </div>
                <div className="text-right">
                  <div className="text-sm font-medium text-emerald-400">
                    ${provider.cost}/mo
                  </div>
                  <div className="text-xs text-slate-500">
                    {provider.requests === 999999 ? 'Unlimited' : `${provider.requests/1000}k`} req
                  </div>
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </div>

      {/* Monthly Trend */}
      <div className="bg-slate-800/30 rounded-lg p-4">
        <h3 className="text-sm font-medium text-slate-300 mb-4">6-Month Cost Trend</h3>
        <div className="h-48">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={monthlyTrend}>
              <XAxis 
                dataKey="month" 
                axisLine={false}
                tickLine={false}
                tick={{ fill: '#94a3b8', fontSize: 12 }}
              />
              <YAxis 
                axisLine={false}
                tickLine={false}
                tick={{ fill: '#94a3b8', fontSize: 12 }}
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: '#1e293b',
                  border: '1px solid #334155',
                  borderRadius: '8px',
                  color: '#f1f5f9'
                }}
              />
              <Bar 
                dataKey="traditional" 
                fill="#ef4444" 
                radius={[2, 2, 0, 0]}
                name="Traditional"
              />
              <Bar 
                dataKey="optimized" 
                fill="#10b981" 
                radius={[2, 2, 0, 0]}
                name="Multi-RPC"
              />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Bottom Stats */}
      <div className="grid grid-cols-3 gap-4 mt-6 pt-4 border-t border-slate-800">
        <div className="text-center">
          <div className="flex items-center justify-center space-x-1 mb-1">
            <TrendingDown className="w-4 h-4 text-emerald-400" />
            <span className="text-lg font-bold text-emerald-400">{savingsPercentage}%</span>
          </div>
          <div className="text-xs text-slate-500">Cost Reduction</div>
        </div>
        <div className="text-center">
          <div className="flex items-center justify-center space-x-1 mb-1">
            <Zap className="w-4 h-4 text-blue-400" />
            <span className="text-lg font-bold text-blue-400">3</span>
          </div>
          <div className="text-xs text-slate-500">FREE Providers</div>
        </div>
        <div className="text-center">
          <div className="flex items-center justify-center space-x-1 mb-1">
            <Target className="w-4 h-4 text-purple-400" />
            <span className="text-lg font-bold text-purple-400">200k+</span>
          </div>
          <div className="text-xs text-slate-500">Free Requests</div>
        </div>
      </div>
    </motion.div>
  )
}
