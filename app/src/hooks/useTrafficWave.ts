import { useState, useEffect, useRef, useCallback } from 'react'
import { coreBridge, type TrafficStats as CoreTrafficStats } from '../../src-tauri/types'

export interface TrafficDataPoint {
  timestamp: number
  uploadSpeed: number  // bytes/sec
  downloadSpeed: number  // bytes/sec
}

export interface TrafficStats {
  totalUpload: number
  totalDownload: number
  blockedCount: number
  currentUploadSpeed: number
  currentDownloadSpeed: number
}

/**
 * 实时流量波形图 Hook
 * 支持 Canvas 渲染、60FPS 更新、平滑插值
 */
export function useTrafficWave(maxPoints: number = 300) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [dataPoints, setDataPoints] = useState<TrafficDataPoint[]>([])
  const [stats, setStats] = useState<TrafficStats>({
    totalUpload: 0,
    totalDownload: 0,
    blockedCount: 0,
    currentUploadSpeed: 0,
    currentDownloadSpeed: 0
  })
  const [isRunning, setIsRunning] = useState(true)
  const animationRef = useRef<number>()

  // Convert CoreTrafficStats to local TrafficStats format
  const convertTrafficStats = (core: CoreTrafficStats): TrafficStats => ({
    totalUpload: core.total_upload,
    totalDownload: core.total_download,
    blockedCount: core.blocked_connections,
    currentUploadSpeed: core.upload_speed,
    currentDownloadSpeed: core.download_speed,
  })

  // 从 Rust Core 获取流量统计
  const fetchStats = useCallback(async (): Promise<TrafficStats> => {
    try {
      const result = await coreBridge.getTrafficStats()
      return convertTrafficStats(result)
    } catch (err) {
      console.error('Failed to fetch traffic stats:', err)
      return stats
    }
  }, [stats])

  // 初始化数据获取
  useEffect(() => {
    const fetchData = async () => {
      const newStats = await fetchStats()
      setStats(newStats)
      
      const newPoint: TrafficDataPoint = {
        timestamp: Date.now(),
        uploadSpeed: newStats.currentUploadSpeed,
        downloadSpeed: newStats.currentDownloadSpeed
      }
      
      setDataPoints(prev => {
        const updated = [...prev, newPoint]
        if (updated.length > maxPoints) {
          return updated.slice(updated.length - maxPoints)
        }
        return updated
      })
    }

    fetchData()
    
    const timer = setInterval(fetchData, 1000)
    return () => clearInterval(timer)
  }, [fetchStats, maxPoints])

  // Canvas 渲染波形图
  const renderWave = useCallback(() => {
    const canvas = canvasRef.current
    if (!canvas || dataPoints.length < 2) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const width = canvas.width
    const height = canvas.height
    
    // 清空画布
    ctx.clearRect(0, 0, width, height)
    
    // 创建渐变背景
    const gradient = ctx.createLinearGradient(0, 0, 0, height)
    gradient.addColorStop(0, 'rgba(15, 17, 21, 0.9)')
    gradient.addColorStop(1, 'rgba(30, 32, 40, 0.8)')
    ctx.fillStyle = gradient
    ctx.fillRect(0, 0, width, height)

    // 找到最大值用于缩放
    const maxValue = Math.max(
      ...dataPoints.map(p => Math.max(p.uploadSpeed, p.downloadSpeed)),
      1
    )

    // 绘制上传波形 (荧光绿)
    ctx.beginPath()
    ctx.strokeStyle = 'rgba(52, 211, 153, 0.9)'
    ctx.lineWidth = 2
    ctx.shadowColor = 'rgba(52, 211, 153, 0.5)'
    ctx.shadowBlur = 10

    dataPoints.forEach((point, index) => {
      const x = (index / (maxPoints - 1)) * width
      const y = height - (point.uploadSpeed / maxValue) * height * 0.8
      
      if (index === 0) {
        ctx.moveTo(x, y)
      } else {
        // 平滑插值
        const prevPoint = dataPoints[index - 1]
        const cpX = x - (width / maxPoints) / 2
        const cpY = height - (prevPoint.uploadSpeed / maxValue) * height * 0.8
        ctx.quadraticCurveTo(cpX, cpY, x, y)
      }
    })
    ctx.stroke()

    // 绘制下载波形 (霓虹蓝)
    ctx.beginPath()
    ctx.strokeStyle = 'rgba(59, 130, 246, 0.9)'
    ctx.shadowColor = 'rgba(59, 130, 246, 0.5)'
    
    dataPoints.forEach((point, index) => {
      const x = (index / (maxPoints - 1)) * width
      const y = height - (point.downloadSpeed / maxValue) * height * 0.8
      
      if (index === 0) {
        ctx.moveTo(x, y)
      } else {
        const prevPoint = dataPoints[index - 1]
        const cpX = x - (width / maxPoints) / 2
        const cpY = height - (prevPoint.downloadSpeed / maxValue) * height * 0.8
        ctx.quadraticCurveTo(cpX, cpY, x, y)
      }
    })
    ctx.stroke()
    
    // 重置阴影
    ctx.shadowBlur = 0

    // 绘制图例
    ctx.font = '12px HarmonyOS Sans SC, Inter, sans-serif'
    ctx.fillStyle = 'rgba(52, 211, 153, 0.9)'
    ctx.fillText('▲ 上传', 10, 20)
    ctx.fillStyle = 'rgba(59, 130, 246, 0.9)'
    ctx.fillText('▼ 下载', 10, 36)

  }, [dataPoints, maxPoints])

  // 动画循环
  useEffect(() => {
    const animate = () => {
      renderWave()
      animationRef.current = requestAnimationFrame(animate)
    }
    
    if (isRunning) {
      animationRef.current = requestAnimationFrame(animate)
    }
    
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current)
      }
    }
  }, [renderWave, isRunning])

  // 调整 Canvas 尺寸
  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    const resizeObserver = new ResizeObserver(entries => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect
        canvas.width = width * window.devicePixelRatio
        canvas.height = height * window.devicePixelRatio
        canvas.style.width = `${width}px`
        canvas.style.height = `${height}px`
        
        const ctx = canvas.getContext('2d')
        if (ctx) {
          ctx.scale(window.devicePixelRatio, window.devicePixelRatio)
        }
      }
    })

    resizeObserver.observe(canvas.parentElement!)
    return () => resizeObserver.disconnect()
  }, [])

  return {
    canvasRef,
    dataPoints,
    stats,
    isRunning,
    setIsRunning,
    refreshStats: fetchStats
  }
}
