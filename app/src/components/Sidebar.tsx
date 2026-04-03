import { motion } from 'framer-motion'
import { clsx } from 'clsx'

interface SidebarProps {
  collapsed: boolean
  onToggle: () => void
  darkMode: boolean
  onThemeToggle: () => void
}

const menuItems = [
  { icon: '📊', label: '仪表盘', path: '/' },
  { icon: '🔍', label: '进程监控', path: '/monitor' },
  { icon: '🧠', label: '规则引擎', path: '/rules' },
  { icon: '⚙️', label: '系统设置', path: '/settings' },
]

export default function Sidebar({ collapsed, onToggle, darkMode, onThemeToggle }: SidebarProps) {
  return (
    <motion.div
      className={clsx(
        'glass-card h-full rounded-2xl shadow-2xl flex flex-col',
        'spring-transition'
      )}
      initial={{ scale: 0.95 }}
      animate={{ scale: 1 }}
      transition={{ type: 'spring', stiffness: 300, damping: 30 }}
    >
      {/* Logo Area */}
      <div className="p-4 border-b border-[var(--border-subtle)]">
        <motion.div
          className="flex items-center gap-3"
          animate={{ justifyContent: collapsed ? 'center' : 'flex-start' }}
        >
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-[var(--primary)] to-blue-600 flex items-center justify-center text-white font-bold text-lg">
            N
          </div>
          {!collapsed && (
            <motion.div
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              className="flex flex-col"
            >
              <span className="font-bold text-lg">NetSentinel</span>
              <span className="text-xs text-[var(--text-secondary)]">v2.0</span>
            </motion.div>
          )}
        </motion.div>
      </div>

      {/* Navigation Menu */}
      <nav className="flex-1 p-3 space-y-1">
        {menuItems.map((item) => (
          <a
            key={item.path}
            href={item.path}
            className={clsx(
              'flex items-center gap-3 px-3 py-2.5 rounded-lg',
              'hover:bg-[var(--primary-glow)] spring-transition cursor-pointer',
              collapsed ? 'justify-center' : ''
            )}
          >
            <span className="text-xl">{item.icon}</span>
            {!collapsed && (
              <span className="text-sm font-medium">{item.label}</span>
            )}
          </a>
        ))}
      </nav>

      {/* Bottom Actions */}
      <div className="p-3 border-t border-[var(--border-subtle)] space-y-1">
        {/* Theme Toggle */}
        <button
          onClick={onThemeToggle}
          className={clsx(
            'w-full flex items-center gap-3 px-3 py-2.5 rounded-lg',
            'hover:bg-[var(--primary-glow)] spring-transition',
            collapsed ? 'justify-center' : ''
          )}
        >
          <span className="text-xl">{darkMode ? '☀️' : '🌙'}</span>
          {!collapsed && <span className="text-sm font-medium">{darkMode ? '浅色模式' : '深色模式'}</span>}
        </button>

        {/* Collapse Toggle */}
        <button
          onClick={onToggle}
          className={clsx(
            'w-full flex items-center gap-3 px-3 py-2.5 rounded-lg',
            'hover:bg-[var(--primary-glow)] spring-transition',
            collapsed ? 'justify-center' : ''
          )}
        >
          <span className="text-xl">{collapsed ? '→' : '←'}</span>
          {!collapsed && <span className="text-sm font-medium">折叠侧边栏</span>}
        </button>
      </div>
    </motion.div>
  )
}
