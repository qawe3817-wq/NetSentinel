import { useState } from 'react'
import { motion } from 'framer-motion'

interface TopBarProps {
  darkMode: boolean
}

export default function TopBar({ darkMode }: TopBarProps) {
  const [searchOpen, setSearchOpen] = useState(false)

  // Handle Ctrl+K for search
  useState(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault()
        setSearchOpen(true)
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  })

  return (
    <motion.header
      initial={{ y: -20, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      transition={{ duration: 0.3 }}
      className="glass-card mx-4 mt-4 mb-2 rounded-xl px-4 py-3 flex items-center justify-between"
    >
      {/* Page Title */}
      <div>
        <h1 className="text-xl font-bold">仪表盘</h1>
        <p className="text-xs text-[var(--text-secondary)]">实时监控网络状态</p>
      </div>

      {/* Right Actions */}
      <div className="flex items-center gap-3">
        {/* Search Button */}
        <button
          onClick={() => setSearchOpen(true)}
          className="btn-click glass-card px-4 py-2 rounded-lg flex items-center gap-2 text-sm"
        >
          <span>🔍</span>
          <span className="text-[var(--text-secondary)]">搜索...</span>
          <kbd className="hidden md:inline-block px-2 py-0.5 bg-[var(--border-subtle)] rounded text-xs">
            Ctrl+K
          </kbd>
        </button>

        {/* Notification Bell */}
        <button className="btn-click relative w-10 h-10 rounded-lg glass-card flex items-center justify-center hover:bg-[var(--primary-glow)] spring-transition">
          <span className="text-xl">🔔</span>
          <span className="absolute top-2 right-2 w-2 h-2 bg-[var(--danger)] rounded-full"></span>
        </button>
      </div>

      {/* Search Modal Overlay */}
      {searchOpen && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 bg-black/50 z-50 flex items-start justify-center pt-20"
          onClick={() => setSearchOpen(false)}
        >
          <motion.div
            initial={{ scale: 0.95, y: -20 }}
            animate={{ scale: 1, y: 0 }}
            exit={{ scale: 0.95, y: -20 }}
            className="glass-card w-full max-w-2xl rounded-2xl p-4"
            onClick={(e) => e.stopPropagation()}
          >
            <input
              type="text"
              placeholder="搜索进程、规则、设置..."
              className="w-full bg-transparent border-none outline-none text-lg p-2"
              autoFocus
            />
            <div className="mt-4 text-sm text-[var(--text-secondary)]">
              <p>💡 提示：使用 Ctrl+K 快速打开搜索</p>
            </div>
          </motion.div>
        </motion.div>
      )}
    </motion.header>
  )
}
