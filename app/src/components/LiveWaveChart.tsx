import { useRef, useEffect } from 'react'
import { useTrafficWave, type TrafficDataPoint } from '../hooks/useTrafficWave'
import { motion } from 'framer-motion'

/**
 * 实时流量波形图组件
 * 基于 Canvas 渲染，支持 60FPS 实时更新
 * 特性：
 * - 荧光绿/霓虹蓝双色波形
 * - 平滑插值曲线
 * - 光晕拖尾效果
 * - 动态缩放
 */
export default function LiveWaveChart() {
  const { canvasRef, stats, isRunning, setIsRunning } = useTrafficWave(300)
  const containerRef = useRef<HTMLDivElement>(null)

  // 格式化速度显示
  const formatSpeed = (bytesPerSec: number): string => {
    if (bytesPerSec >= 1024 * 1024) {
      return `${(bytesPerSec / (1024 * 1024)).toFixed(2)} MB/s`
    } else if (bytesPerSec >= 1024) {
      return `${(bytesPerSec / 1024).toFixed(2)} KB/s`
    }
    return `${bytesPerSec} B/s`
  }

  return (
    <motion.div
      initial={{ y: 20, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      transition={{ delay: 0.1, type: 'spring', stiffness: 100, damping: 20 }}
      className="glass-card rounded-2xl p-6 col-span-2"
    >
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">实时流量波形</h3>
        
        <div className="flex items-center gap-4 text-sm">
          <div className="flex items-center gap-2">
            <span className="w-3 h-3 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]"></span>
            <span className="text-[var(--text-secondary)]">上传</span>
            <span className="font-mono font-medium">{formatSpeed(stats.currentUploadSpeed)}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="w-3 h-3 rounded-full bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.6)]"></span>
            <span className="text-[var(--text-secondary)]">下载</span>
            <span className="font-mono font-medium">{formatSpeed(stats.currentDownloadSpeed)}</span>
          </div>
          
          <button
            onClick={() => setIsRunning(!isRunning)}
            className="btn-click px-3 py-1.5 glass-card rounded-lg text-xs hover:bg-[var(--primary-glow)] spring-transition"
          >
            {isRunning ? '⏸️ 暂停' : '▶️ 继续'}
          </button>
        </div>
      </div>

      {/* Canvas 渲染区域 */}
      <div 
        ref={containerRef}
        className="h-48 rounded-xl overflow-hidden relative"
        style={{
          background: 'linear-gradient(180deg, rgba(15,17,21,0.9) 0%, rgba(30,32,40,0.8) 100%)'
        }}
      >
        <canvas
          ref={canvasRef}
          className="w-full h-full"
          style={{ display: 'block' }}
        />
        
        {/* 网格背景装饰 */}
        <div 
          className="absolute inset-0 pointer-events-none opacity-20"
          style={{
            backgroundImage: `
              linear-gradient(rgba(255,255,255,0.05) 1px, transparent 1px),
              linear-gradient(90deg, rgba(255,255,255,0.05) 1px, transparent 1px)
            `,
            backgroundSize: '40px 40px'
          }}
        />
      </div>

      {/* 统计信息 */}
      <div className="mt-4 grid grid-cols-3 gap-4 text-sm">
        <div className="p-3 rounded-lg bg-[var(--border-subtle)]">
          <div className="text-xs text-[var(--text-secondary)] mb-1">总上传</div>
          <div className="font-mono font-medium text-green-500">
            {formatSpeed(stats.totalUpload)}
          </div>
        </div>
        <div className="p-3 rounded-lg bg-[var(--border-subtle)]">
          <div className="text-xs text-[var(--text-secondary)] mb-1">总下载</div>
          <div className="font-mono font-medium text-blue-500">
            {formatSpeed(stats.totalDownload)}
          </div>
        </div>
        <div className="p-3 rounded-lg bg-[var(--border-subtle)]">
          <div className="text-xs text-[var(--text-secondary)] mb-1">已阻断连接</div>
          <div className="font-mono font-medium text-[var(--danger)]">
            {stats.blockedCount}
          </div>
        </div>
      </div>
    </motion.div>
  )
}
