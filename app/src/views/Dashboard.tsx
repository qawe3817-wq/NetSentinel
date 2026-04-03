import { motion } from 'framer-motion'
import LiveWaveChart from '../components/LiveWaveChart'

export default function Dashboard() {
  return (
    <div className="space-y-6">
      {/* Hero Card - Core Status */}
      <motion.div
        initial={{ scale: 0.95, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ type: 'spring', stiffness: 200, damping: 20 }}
        className="glass-card rounded-2xl p-8 relative overflow-hidden"
      >
        {/* Dynamic Mesh Gradient Background */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-500/10 via-purple-500/10 to-pink-500/10"></div>
        
        <div className="relative z-10 flex items-center justify-between">
          <div>
            <h2 className="text-3xl font-bold mb-2">安全</h2>
            <p className="text-[var(--text-secondary)]">当前网络状态正常，未发现异常上传</p>
            
            <div className="mt-6 flex gap-4">
              <button className="btn-click px-6 py-3 bg-[var(--primary)] text-white rounded-xl font-medium shadow-lg shadow-[var(--primary-glow)]">
                🛡️ 防护模式：静默
              </button>
              <button className="btn-click px-6 py-3 glass-card rounded-xl font-medium">
                ⚡ 快速阻断
              </button>
            </div>
          </div>

          {/* Circular Progress */}
          <div className="relative w-40 h-40">
            <svg className="w-full h-full transform -rotate-90">
              <circle
                cx="80"
                cy="80"
                r="70"
                stroke="var(--border-subtle)"
                strokeWidth="12"
                fill="none"
              />
              <circle
                cx="80"
                cy="80"
                r="70"
                stroke="var(--primary)"
                strokeWidth="12"
                fill="none"
                strokeDasharray="440"
                strokeDashoffset="110"
                strokeLinecap="round"
                className="spring-transition"
              />
            </svg>
            <div className="absolute inset-0 flex flex-col items-center justify-center">
              <span className="text-3xl font-bold">25%</span>
              <span className="text-xs text-[var(--text-secondary)]">上传占用</span>
            </div>
          </div>
        </div>
      </motion.div>

      {/* Live Wave Chart & Threat Intel */}
      <div className="grid grid-cols-3 gap-6">
        {/* Real-time Traffic Wave - Using Canvas Component */}
        <LiveWaveChart />

        {/* Threat Intelligence */}
        <motion.div
          initial={{ y: 20, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ delay: 0.2 }}
          className="glass-card rounded-2xl p-6"
        >
          <h3 className="text-lg font-semibold mb-4">威胁情报</h3>
          <div className="space-y-3">
            {[
              { process: 'video.exe', ip: '192.168.1.100', time: '2 分钟前' },
              { process: 'download.dll', ip: '10.0.0.55', time: '15 分钟前' },
              { process: 'unknown.sys', ip: '172.16.0.88', time: '32 分钟前' },
            ].map((threat, i) => (
              <div
                key={i}
                className="p-3 rounded-lg bg-[var(--border-subtle)] hover:bg-[var(--danger-pulse)] spring-transition cursor-pointer"
              >
                <div className="flex justify-between items-start">
                  <span className="font-mono text-sm">{threat.process}</span>
                  <span className="text-xs text-[var(--text-secondary)]">{threat.time}</span>
                </div>
                <div className="text-xs text-[var(--text-secondary)] mt-1">
                  → {threat.ip}
                </div>
              </div>
            ))}
          </div>
          <button className="w-full mt-4 py-2 text-sm text-[var(--primary)] hover:underline">
            查看全部 →
          </button>
        </motion.div>
      </div>
    </div>
  )
}
