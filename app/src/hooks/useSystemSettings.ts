import { useState, useEffect } from 'react'

export interface SystemSettings {
  // 通用设置
  autoStart: boolean
  silentMode: boolean
  watchdogEnabled: boolean
  
  // 网络适配器
  networkAdapters: NetworkAdapter[]
  
  // 外观
  theme: ThemeConfig
}

export interface NetworkAdapter {
  id: string
  name: string
  type: 'ethernet' | 'wifi' | 'virtual'
  status: 'active' | 'disconnected'
  policy?: string
}

export interface ThemeConfig {
  mode: 'light' | 'dark' | 'system'
  preset: 'geek-black' | 'pure-white' | 'cyberpunk' | 'custom'
  primaryColor: string
}

/**
 * 系统设置 Hook
 * 管理所有应用配置项
 */
export function useSystemSettings() {
  const [settings, setSettings] = useState<SystemSettings>({
    autoStart: true,
    silentMode: false,
    watchdogEnabled: true,
    networkAdapters: [
      {
        id: 'adapter-1',
        name: 'Wi-Fi (Intel AX200)',
        type: 'wifi',
        status: 'active'
      },
      {
        id: 'adapter-2',
        name: 'Ethernet (Realtek PCIe)',
        type: 'ethernet',
        status: 'disconnected'
      },
      {
        id: 'adapter-3',
        name: 'VMware Network Adapter',
        type: 'virtual',
        status: 'active'
      }
    ],
    theme: {
      mode: 'system',
      preset: 'geek-black',
      primaryColor: '#60A5FA'
    }
  })

  const [startupItems, setStartupItems] = useState([
    {
      id: 'startup-1',
      name: 'NetSentinel Core',
      publisher: 'NetSentinel Team',
      verified: true,
      enabled: true
    },
    {
      id: 'startup-2',
      name: 'Video Client',
      publisher: 'Unknown',
      verified: false,
      enabled: true
    },
    {
      id: 'startup-3',
      name: 'Cloud Drive',
      publisher: 'Tech Corp',
      verified: true,
      enabled: false
    }
  ])

  // 从本地存储加载设置（实际应从配置文件读取）
  useEffect(() => {
    try {
      const saved = localStorage.getItem('netsentinel-settings')
      if (saved) {
        setSettings(JSON.parse(saved))
      }
    } catch (error) {
      console.error('Failed to load settings:', error)
    }
  }, [])

  // 保存设置到本地存储
  useEffect(() => {
    try {
      localStorage.setItem('netsentinel-settings', JSON.stringify(settings))
    } catch (error) {
      console.error('Failed to save settings:', error)
    }
  }, [settings])

  // 更新通用设置
  const updateGeneralSetting = useCallback((key: keyof Pick<SystemSettings, 'autoStart' | 'silentMode' | 'watchdogEnabled'>, value: boolean) => {
    setSettings(prev => ({ ...prev, [key]: value }))
  }, [])

  // 更新网络适配器策略
  const updateAdapterPolicy = useCallback((adapterId: string, policy: string) => {
    setSettings(prev => ({
      ...prev,
      networkAdapters: prev.networkAdapters.map(adapter =>
        adapter.id === adapterId ? { ...adapter, policy } : adapter
      )
    }))
  }, [])

  // 更新主题
  const updateTheme = useCallback((theme: Partial<ThemeConfig>) => {
    setSettings(prev => ({
      ...prev,
      theme: { ...prev.theme, ...theme }
    }))
  }, [])

  // 切换启动项
  const toggleStartupItem = useCallback((itemId: string) => {
    setStartupItems(prev =>
      prev.map(item =>
        item.id === itemId ? { ...item, enabled: !item.enabled } : item
      )
    )
  }, [])

  // 应用预设主题
  const applyPresetTheme = useCallback((preset: ThemeConfig['preset']) => {
    const presets = {
      'geek-black': {
        mode: 'dark' as const,
        primaryColor: '#60A5FA'
      },
      'pure-white': {
        mode: 'light' as const,
        primaryColor: '#3B82F6'
      },
      'cyberpunk': {
        mode: 'dark' as const,
        primaryColor: '#F0ABFC'
      }
    }

    setSettings(prev => ({
      ...prev,
      theme: {
        ...prev.theme,
        preset,
        ...presets[preset]
      }
    }))
  }, [])

  return {
    settings,
    startupItems,
    updateGeneralSetting,
    updateAdapterPolicy,
    updateTheme,
    toggleStartupItem,
    applyPresetTheme
  }
}
