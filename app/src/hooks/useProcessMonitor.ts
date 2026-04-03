import { useState, useEffect, useCallback } from 'react'
import { coreBridge, type ProcessInfo as CoreProcessInfo } from '../../src-tauri/types'

export interface ProcessInfo {
  pid: number
  name: string
  path: string
  uploadSpeed: number
  downloadSpeed: number
  connections: number
  status: 'normal' | 'suspicious' | 'blocked' | 'whitelisted'
  signature?: {
    verified: boolean
    publisher?: string
  }
  parentPid?: number
  children?: number[]
}

export interface FilterOption {
  id: string
  label: string
  active: boolean
}

/**
 * 进程监控数据 Hook
 * 支持虚拟滚动、智能过滤、批量操作
 * 通过 Tauri IPC 与 Rust Core 服务通信
 */
export function useProcessMonitor() {
  const [processes, setProcesses] = useState<ProcessInfo[]>([])
  const [selectedPids, setSelectedPids] = useState<number[]>([])
  const [filters, setFilters] = useState<FilterOption[]>([
    { id: 'high-upload', label: '高上传', active: false },
    { id: 'many-connections', label: '多连接', active: false },
    { id: 'unknown-signature', label: '未知签名', active: false },
    { id: 'outside-whitelist', label: '白名单外', active: false },
  ])
  const [searchQuery, setSearchQuery] = useState('')
  const [sortConfig, setSortConfig] = useState<{
    key: keyof ProcessInfo
    direction: 'asc' | 'desc'
  } | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Convert CoreProcessInfo to local ProcessInfo format
  const convertProcessInfo = (core: CoreProcessInfo): ProcessInfo => ({
    pid: core.pid,
    name: core.name,
    path: core.path,
    uploadSpeed: core.upload_speed,
    downloadSpeed: core.download_speed,
    connections: core.connection_count,
    status: core.risk_score > 0.8 ? 'suspicious' : core.is_signed ? 'normal' : 'blocked',
    signature: {
      verified: core.is_signed,
      publisher: core.is_signed ? 'Verified Publisher' : undefined,
    },
    parentPid: core.parent_pid,
  })

  // 从 Rust Core 服务获取进程列表
  const fetchProcesses = useCallback(async () => {
    try {
      setLoading(true)
      // 调用 Rust Core 的 get_processes() IPC 接口
      const result = await coreBridge.getProcesses()
      setProcesses(result.map(convertProcessInfo))
      setError(null)
    } catch (err) {
      console.error('Failed to fetch processes:', err)
      setError(err instanceof Error ? err.message : 'Unknown error')
    } finally {
      setLoading(false)
    }
  }, [])

  // 初始化数据
  useEffect(() => {
    fetchProcesses()
    
    // 定时刷新（实际应从 Core 服务通过事件订阅）
    const timer = setInterval(fetchProcesses, 2000)

    return () => clearInterval(timer)
  }, [fetchProcesses])

  // 过滤逻辑
  const filteredProcesses = processes.filter(proc => {
    // 搜索过滤
    if (searchQuery && !proc.name.toLowerCase().includes(searchQuery.toLowerCase())) {
      return false
    }

    // 标签过滤
    for (const filter of filters) {
      if (!filter.active) continue

      switch (filter.id) {
        case 'high-upload':
          if (proc.uploadSpeed < 1 * 1024 * 1024) return false
          break
        case 'many-connections':
          if (proc.connections < 50) return false
          break
        case 'unknown-signature':
          if (proc.signature?.verified) return false
          break
        case 'outside-whitelist':
          if (proc.status === 'whitelisted') return false
          break
      }
    }

    return true
  })

  // 排序逻辑
  const sortedProcesses = sortConfig
    ? [...filteredProcesses].sort((a, b) => {
        const aValue = a[sortConfig.key!]
        const bValue = b[sortConfig.key!]
        
        if (typeof aValue === 'string' && typeof bValue === 'string') {
          return sortConfig.direction === 'asc'
            ? aValue.localeCompare(bValue)
            : bValue.localeCompare(aValue)
        }
        
        if (typeof aValue === 'number' && typeof bValue === 'number') {
          return sortConfig.direction === 'asc' ? aValue - bValue : bValue - aValue
        }
        
        return 0
      })
    : filteredProcesses

  // 选择/取消选择进程
  const toggleSelection = useCallback((pid: number) => {
    setSelectedPids(prev =>
      prev.includes(pid)
        ? prev.filter(id => id !== pid)
        : [...prev, pid]
    )
  }, [])

  const selectAll = useCallback(() => {
    setSelectedPids(filteredProcesses.map(p => p.pid))
  }, [filteredProcesses])

  const clearSelection = useCallback(() => {
    setSelectedPids([])
  }, [])

  // 批量操作 - 调用 Rust Core
  const bulkBlock = useCallback(async () => {
    console.log('Blocking processes:', selectedPids)
    try {
      for (const pid of selectedPids) {
        await coreBridge.blockProcess(pid, 300)
      }
      await fetchProcesses()
    } catch (err) {
      console.error('Failed to block processes:', err)
    }
    clearSelection()
  }, [selectedPids, clearSelection, fetchProcesses])

  const bulkWhitelist = useCallback(async () => {
    console.log('Whitelisting processes:', selectedPids)
    try {
      for (const proc of processes.filter(p => selectedPids.includes(p.pid))) {
        await coreBridge.addToWhitelist(proc.path)
      }
      await fetchProcesses()
    } catch (err) {
      console.error('Failed to whitelist processes:', err)
    }
    clearSelection()
  }, [selectedPids, processes, clearSelection, fetchProcesses])

  // 切换过滤器
  const toggleFilter = useCallback((filterId: string) => {
    setFilters(prev =>
      prev.map(f =>
        f.id === filterId ? { ...f, active: !f.active } : f
      )
    )
  }, [])

  // 排序处理
  const requestSort = useCallback((key: keyof ProcessInfo) => {
    setSortConfig(prev => {
      if (prev?.key === key) {
        return prev.direction === 'asc'
          ? { key, direction: 'desc' }
          : null
      }
      return { key, direction: 'desc' }
    })
  }, [])

  // 结束进程 - 调用 Rust Core
  const killProcess = useCallback(async (pid: number) => {
    console.log('Killing process:', pid)
    try {
      await coreBridge.terminateProcess(pid)
      await fetchProcesses()
    } catch (err) {
      console.error('Failed to kill process:', err)
    }
  }, [fetchProcesses])

  // 阻断进程 - 调用 Rust Core
  const blockProcess = useCallback(async (pid: number, durationSecs: number = 300) => {
    console.log('Blocking process:', pid)
    try {
      await coreBridge.blockProcess(pid, durationSecs)
      await fetchProcesses()
    } catch (err) {
      console.error('Failed to block process:', err)
    }
  }, [fetchProcesses])

  return {
    processes: sortedProcesses,
    selectedPids,
    filters,
    searchQuery,
    setSearchQuery,
    toggleSelection,
    selectAll,
    clearSelection,
    bulkBlock,
    bulkWhitelist,
    toggleFilter,
    requestSort,
    killProcess,
    blockProcess,
    totalCount: processes.length,
    filteredCount: sortedProcesses.length,
    loading,
    error
  }
}
