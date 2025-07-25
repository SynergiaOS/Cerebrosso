'use client'

import { motion } from 'framer-motion'
import { Shield, AlertTriangle, TrendingDown, Activity } from 'lucide-react'

export function RiskAnalysis() {
  const riskMetrics = [
    { 
      name: 'Portfolio Risk', 
      value: 23, 
      threshold: 30, 
      status: 'low',
      description: 'Current exposure within limits'
    },
    { 
      name: 'Slippage Risk', 
      value: 45, 
      threshold: 50, 
      status: 'medium',
      description: 'Moderate slippage detected'
    },
    { 
      name: 'Liquidity Risk', 
      value: 12, 
      threshold: 25, 
      status: 'low',
      description: 'Sufficient liquidity available'
    },
    { 
      name: 'MEV Risk', 
      value: 67, 
      threshold: 70, 
      status: 'high',
      description: 'High MEV competition detected'
    },
  ]

  const getRiskColor = (status: string) => {
    switch (status) {
      case 'low': return 'text-emerald-500'
      case 'medium': return 'text-yellow-500'
      case 'high': return 'text-red-500'
      default: return 'text-slate-500'
    }
  }

  const getRiskBg = (status: string) => {
    switch (status) {
      case 'low': return 'bg-emerald-500'
      case 'medium': return 'bg-yellow-500'
      case 'high': return 'bg-red-500'
      default: return 'bg-slate-500'
    }
  }

  const overallRisk = Math.round(riskMetrics.reduce((sum, metric) => sum + metric.value, 0) / riskMetrics.length)

  return (
    <div className="bg-slate-900/50 backdrop-blur-sm rounded-xl border border-slate-800 p-6">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-lg font-semibold text-white flex items-center">
          <Shield className="w-5 h-5 mr-2 text-blue-500" />
          Risk Analysis
        </h3>
        <div className={`flex items-center space-x-2 ${
          overallRisk < 30 ? 'text-emerald-500' : 
          overallRisk < 60 ? 'text-yellow-500' : 'text-red-500'
        }`}>
          <Activity className="w-4 h-4" />
          <span className="text-sm font-medium">{overallRisk}% Risk</span>
        </div>
      </div>

      <div className="space-y-4">
        {riskMetrics.map((metric, index) => (
          <motion.div
            key={metric.name}
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: index * 0.1 }}
            className="p-4 rounded-lg bg-slate-800/50 border border-slate-700"
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center space-x-3">
                <div className={`w-2 h-2 rounded-full ${getRiskBg(metric.status)}`} />
                <span className="font-medium text-slate-200">{metric.name}</span>
              </div>
              <span className={`text-sm font-medium ${getRiskColor(metric.status)}`}>
                {metric.value}%
              </span>
            </div>

            <div className="mb-2">
              <div className="w-full h-2 bg-slate-700 rounded-full overflow-hidden">
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: `${metric.value}%` }}
                  transition={{ delay: index * 0.1 + 0.2, duration: 0.5 }}
                  className={`h-full rounded-full ${getRiskBg(metric.status)}`}
                />
              </div>
              <div className="flex justify-between text-xs text-slate-500 mt-1">
                <span>0%</span>
                <span className="text-slate-400">Threshold: {metric.threshold}%</span>
                <span>100%</span>
              </div>
            </div>

            <p className="text-xs text-slate-400">{metric.description}</p>
          </motion.div>
        ))}
      </div>

      {/* Overall Risk Summary */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.5 }}
        className={`mt-6 p-4 rounded-lg border ${
          overallRisk < 30 ? 'bg-emerald-500/10 border-emerald-500/20' :
          overallRisk < 60 ? 'bg-yellow-500/10 border-yellow-500/20' :
          'bg-red-500/10 border-red-500/20'
        }`}
      >
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            {overallRisk < 30 ? (
              <Shield className="w-5 h-5 text-emerald-500" />
            ) : overallRisk < 60 ? (
              <AlertTriangle className="w-5 h-5 text-yellow-500" />
            ) : (
              <TrendingDown className="w-5 h-5 text-red-500" />
            )}
            <span className={`font-medium ${
              overallRisk < 30 ? 'text-emerald-400' :
              overallRisk < 60 ? 'text-yellow-400' :
              'text-red-400'
            }`}>
              {overallRisk < 30 ? 'Low Risk' :
               overallRisk < 60 ? 'Medium Risk' :
               'High Risk'}
            </span>
          </div>
          <span className={`text-sm ${
            overallRisk < 30 ? 'text-emerald-300' :
            overallRisk < 60 ? 'text-yellow-300' :
            'text-red-300'
          }`}>
            Overall: {overallRisk}%
          </span>
        </div>
      </motion.div>
    </div>
  )
}
