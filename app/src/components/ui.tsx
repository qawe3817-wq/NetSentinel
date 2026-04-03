import { motion } from 'framer-motion'
import { clsx } from 'clsx'

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost'
  size?: 'sm' | 'md' | 'lg'
  loading?: boolean
  leftIcon?: React.ReactNode
  rightIcon?: React.ReactNode
}

/**
 * 基础按钮组件
 * 支持弹簧动画、多种变体和尺寸
 */
export function Button({
  children,
  variant = 'primary',
  size = 'md',
  loading = false,
  leftIcon,
  rightIcon,
  className,
  disabled,
  ...props
}: ButtonProps) {
  const baseStyles = 'btn-click inline-flex items-center justify-center font-medium rounded-xl spring-transition focus:outline-none focus:ring-2 focus:ring-offset-2'
  
  const variantStyles = {
    primary: 'bg-[var(--primary)] text-white shadow-lg shadow-[var(--primary-glow)] hover:opacity-90 focus:ring-[var(--primary)]',
    secondary: 'glass-card text-[var(--text-main)] hover:bg-[var(--primary-glow)] focus:ring-[var(--primary)]',
    danger: 'bg-[var(--danger)] text-white shadow-lg shadow-[var(--danger-pulse)] hover:opacity-90 focus:ring-[var(--danger)]',
    ghost: 'text-[var(--text-main)] hover:bg-[var(--border-subtle)] focus:ring-[var(--text-secondary)]'
  }
  
  const sizeStyles = {
    sm: 'px-3 py-1.5 text-sm gap-1.5',
    md: 'px-4 py-2.5 text-sm gap-2',
    lg: 'px-6 py-3 text-base gap-2.5'
  }

  return (
    <motion.button
      whileTap={{ scale: 0.96 }}
      className={clsx(
        baseStyles,
        variantStyles[variant],
        sizeStyles[size],
        (disabled || loading) && 'opacity-50 cursor-not-allowed',
        className
      )}
      disabled={disabled || loading}
      {...props}
    >
      {loading && (
        <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
        </svg>
      )}
      {!loading && leftIcon && <span className="flex-shrink-0">{leftIcon}</span>}
      {children}
      {!loading && rightIcon && <span className="flex-shrink-0">{rightIcon}</span>}
    </motion.button>
  )
}

interface CardProps {
  children: React.ReactNode
  className?: string
  hoverable?: boolean
  onClick?: () => void
}

/**
 * 卡片组件
 * 支持毛玻璃效果和悬停交互
 */
export function Card({ children, className, hoverable = false, onClick }: CardProps) {
  return (
    <motion.div
      whileHover={hoverable ? { scale: 1.01 } : undefined}
      whileTap={onClick ? { scale: 0.99 } : undefined}
      onClick={onClick}
      className={clsx(
        'glass-card rounded-2xl p-6',
        hoverable && 'cursor-pointer spring-transition',
        className
      )}
    >
      {children}
    </motion.div>
  )
}

interface ToggleProps {
  checked: boolean
  onChange: (checked: boolean) => void
  label?: string
  description?: string
  disabled?: boolean
}

/**
 * 开关组件
 * 支持标签和描述文本
 */
export function Toggle({ checked, onChange, label, description, disabled = false }: ToggleProps) {
  return (
    <label className={clsx('flex items-center justify-between cursor-pointer', disabled && 'opacity-50 cursor-not-allowed')}>
      <div className="flex-1 pr-4">
        {label && <div className="font-medium text-sm">{label}</div>}
        {description && <div className="text-xs text-[var(--text-secondary)] mt-0.5">{description}</div>}
      </div>
      <div className="relative">
        <input
          type="checkbox"
          checked={checked}
          onChange={(e) => onChange(e.target.checked)}
          disabled={disabled}
          className="sr-only peer"
        />
        <div className="w-11 h-6 bg-gray-300 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--primary)] peer-checked:after:border-white" />
      </div>
    </label>
  )
}

interface BadgeProps {
  children: React.ReactNode
  variant?: 'success' | 'warning' | 'danger' | 'info' | 'neutral'
  size?: 'sm' | 'md'
}

/**
 * 徽章组件
 * 用于状态标识
 */
export function Badge({ children, variant = 'neutral', size = 'md' }: BadgeProps) {
  const variantStyles = {
    success: 'bg-green-500/20 text-green-500',
    warning: 'bg-yellow-500/20 text-yellow-500',
    danger: 'bg-red-500/20 text-red-500',
    info: 'bg-blue-500/20 text-blue-500',
    neutral: 'bg-[var(--border-subtle)] text-[var(--text-secondary)]'
  }

  const sizeStyles = {
    sm: 'px-1.5 py-0.5 text-xs',
    md: 'px-2 py-1 text-sm'
  }

  return (
    <span className={clsx(
      'inline-flex items-center font-medium rounded-full',
      variantStyles[variant],
      sizeStyles[size]
    )}>
      {children}
    </span>
  )
}

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string
  error?: string
  leftAddon?: React.ReactNode
  rightAddon?: React.ReactNode
}

/**
 * 输入框组件
 * 支持标签、错误状态和前后缀
 */
export function Input({
  label,
  error,
  leftAddon,
  rightAddon,
  className,
  ...props
}: InputProps) {
  return (
    <div className="w-full">
      {label && (
        <label className="block text-sm font-medium mb-1.5">
          {label}
        </label>
      )}
      <div className="relative">
        {leftAddon && (
          <div className="absolute left-3 top-1/2 -translate-y-1/2 flex items-center pointer-events-none">
            {leftAddon}
          </div>
        )}
        <input
          className={clsx(
            'w-full px-4 py-2.5 bg-white dark:bg-gray-800 rounded-lg border outline-none spring-transition',
            'border-[var(--border-subtle)] focus:border-[var(--primary)]',
            error && 'border-[var(--danger)] focus:border-[var(--danger)]',
            leftAddon && 'pl-10',
            rightAddon && 'pr-10',
            className
          )}
          {...props}
        />
        {rightAddon && (
          <div className="absolute right-3 top-1/2 -translate-y-1/2 flex items-center">
            {rightAddon}
          </div>
        )}
      </div>
      {error && (
        <p className="mt-1 text-xs text-[var(--danger)]">{error}</p>
      )}
    </div>
  )
}

interface SelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
  label?: string
  options: { value: string; label: string }[]
}

/**
 * 选择框组件
 */
export function Select({ label, options, className, ...props }: SelectProps) {
  return (
    <div className="w-full">
      {label && (
        <label className="block text-sm font-medium mb-1.5">
          {label}
        </label>
      )}
      <select
        className={clsx(
          'w-full px-4 py-2.5 bg-white dark:bg-gray-800 rounded-lg border outline-none spring-transition',
          'border-[var(--border-subtle)] focus:border-[var(--primary)]',
          className
        )}
        {...props}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </div>
  )
}

interface ProgressBarProps {
  value: number
  max?: number
  variant?: 'primary' | 'success' | 'danger'
  showLabel?: boolean
  size?: 'sm' | 'md' | 'lg'
}

/**
 * 进度条组件
 */
export function ProgressBar({
  value,
  max = 100,
  variant = 'primary',
  showLabel = false,
  size = 'md'
}: ProgressBarProps) {
  const percentage = Math.min(100, Math.max(0, (value / max) * 100))
  
  const variantColors = {
    primary: 'bg-[var(--primary)]',
    success: 'bg-green-500',
    danger: 'bg-[var(--danger)]'
  }

  const sizeStyles = {
    sm: 'h-1.5',
    md: 'h-2.5',
    lg: 'h-4'
  }

  return (
    <div className="w-full">
      {showLabel && (
        <div className="flex justify-between text-xs mb-1">
          <span>{value}</span>
          <span>{Math.round(percentage)}%</span>
        </div>
      )}
      <div className={clsx('w-full bg-[var(--border-subtle)] rounded-full overflow-hidden', sizeStyles[size])}>
        <motion.div
          initial={{ width: 0 }}
          animate={{ width: `${percentage}%` }}
          transition={{ type: 'spring', stiffness: 100, damping: 20 }}
          className={clsx('h-full rounded-full', variantColors[variant])}
        />
      </div>
    </div>
  )
}
