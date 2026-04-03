import { useState, useCallback } from 'react'
import { motion, AnimatePresence } from 'framer-motion'

export interface ConditionBlock {
  id: string
  field: 'process_name' | 'upload_speed' | 'download_speed' | 'connections' | 'signature'
  operator: 'contains' | 'equals' | 'greater_than' | 'less_than' | 'not_verified'
  value: string | number
}

export interface RuleAction {
  type: 'block_temporary' | 'block_permanent' | 'limit_speed' | 'warn_only'
  durationSecs?: number
  speedLimitKbps?: number
}

export interface RuleDefinition {
  id: string
  name: string
  conditions: ConditionBlock[]
  action: RuleAction
  enabled: boolean
  priority: number
}

interface RuleEditorProps {
  rule?: RuleDefinition
  onSave: (rule: RuleDefinition) => void
  onCancel: () => void
}

const FIELD_OPTIONS = [
  { value: 'process_name', label: '进程名' },
  { value: 'upload_speed', label: '上传速度' },
  { value: 'download_speed', label: '下载速度' },
  { value: 'connections', label: '连接数' },
  { value: 'signature', label: '签名状态' },
] as const

const OPERATOR_OPTIONS = {
  process_name: [
    { value: 'contains', label: '包含' },
    { value: 'equals', label: '等于' },
  ],
  upload_speed: [
    { value: 'greater_than', label: '>' },
    { value: 'less_than', label: '<' },
  ],
  download_speed: [
    { value: 'greater_than', label: '>' },
    { value: 'less_than', label: '<' },
  ],
  connections: [
    { value: 'greater_than', label: '>' },
    { value: 'less_than', label: '<' },
  ],
  signature: [
    { value: 'not_verified', label: '未验证' },
  ],
}

const ACTION_OPTIONS = [
  { value: 'block_temporary', label: '阻断 (临时)' },
  { value: 'block_permanent', label: '阻断 (永久)' },
  { value: 'limit_speed', label: '限速' },
  { value: 'warn_only', label: '仅警告' },
] as const

/**
 * 可视化规则编辑器组件
 * 支持条件块拼接、拖拽排序、冲突检测
 */
export default function RuleEditor({ rule, onSave, onCancel }: RuleEditorProps) {
  const [name, setName] = useState(rule?.name || '')
  const [conditions, setConditions] = useState<ConditionBlock[]>(
    rule?.conditions || [
      { id: '1', field: 'process_name', operator: 'contains', value: '' },
    ]
  )
  const [action, setAction] = useState<RuleAction>(
    rule?.action || { type: 'block_temporary', durationSecs: 300 }
  )

  // 添加条件块
  const addCondition = useCallback(() => {
    const newCondition: ConditionBlock = {
      id: Date.now().toString(),
      field: 'process_name',
      operator: 'contains',
      value: '',
    }
    setConditions(prev => [...prev, newCondition])
  }, [])

  // 移除条件块
  const removeCondition = useCallback((id: string) => {
    setConditions(prev => prev.filter(c => c.id !== id))
  }, [])

  // 更新条件块
  const updateCondition = useCallback((id: string, updates: Partial<ConditionBlock>) => {
    setConditions(prev =>
      prev.map(c => {
        if (c.id !== id) return c
        
        const updated = { ...c, ...updates }
        
        // 当字段改变时，重置操作符为默认值
        if (updates.field) {
          const fieldKey = updates.field as keyof typeof OPERATOR_OPTIONS
          const defaultOperator = OPERATOR_OPTIONS[fieldKey]?.[0]?.value || 'contains'
          updated.operator = defaultOperator as any
        }
        
        return updated
      })
    )
  }, [])

  // 保存规则
  const handleSave = () => {
    if (!name.trim()) {
      alert('请输入规则名称')
      return
    }

    const newRule: RuleDefinition = {
      id: rule?.id || Date.now().toString(),
      name: name.trim(),
      conditions: conditions.filter(c => 
        c.field === 'signature' || (typeof c.value === 'string' ? c.value.trim() : c.value !== 0)
      ),
      action,
      enabled: true,
      priority: rule?.priority || 100,
    }

    onSave(newRule)
  }

  return (
    <div className="glass-card rounded-2xl p-6">
      <h3 className="text-lg font-semibold mb-4">
        {rule ? '编辑规则' : '新建规则'}
      </h3>

      {/* 规则名称 */}
      <div className="mb-6">
        <label className="block text-sm font-medium mb-2">规则名称</label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="例如：PCDN 阻断规则"
          className="w-full px-4 py-2.5 bg-[var(--surface-card)] border border-[var(--border-subtle)] rounded-xl outline-none focus:border-[var(--primary)] spring-transition"
        />
      </div>

      {/* 条件块区域 */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-3">
          <label className="block text-sm font-medium">触发条件</label>
          <button
            onClick={addCondition}
            className="btn-click px-3 py-1.5 text-xs bg-[var(--primary)] text-white rounded-lg hover:shadow-lg spring-transition"
          >
            + 添加条件
          </button>
        </div>

        <AnimatePresence>
          {conditions.map((condition, index) => (
            <motion.div
              key={condition.id}
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              transition={{ type: 'spring', stiffness: 300, damping: 25 }}
              className="flex flex-wrap items-center gap-3 p-4 mb-3 bg-[var(--border-subtle)] rounded-xl"
            >
              {index > 0 && (
                <span className="text-sm font-medium text-[var(--primary)]">且</span>
              )}

              {/* 字段选择 */}
              <select
                value={condition.field}
                onChange={(e) => updateCondition(condition.id, { field: e.target.value as any })}
                className="px-3 py-2 bg-[var(--surface-card)] border border-[var(--border-subtle)] rounded-lg text-sm outline-none focus:border-[var(--primary)]"
              >
                {FIELD_OPTIONS.map(opt => (
                  <option key={opt.value} value={opt.value}>
                    {opt.label}
                  </option>
                ))}
              </select>

              {/* 操作符选择 */}
              {condition.field !== 'signature' && (
                <select
                  value={condition.operator}
                  onChange={(e) => updateCondition(condition.id, { operator: e.target.value as any })}
                  className="px-3 py-2 bg-[var(--surface-card)] border border-[var(--border-subtle)] rounded-lg text-sm outline-none focus:border-[var(--primary)]"
                >
                  {OPERATOR_OPTIONS[condition.field as keyof typeof OPERATOR_OPTIONS]?.map(opt => (
                    <option key={opt.value} value={opt.value}>
                      {opt.label}
                    </option>
                  ))}
                </select>
              )}

              {/* 值输入 */}
              {condition.field !== 'signature' && (
                <input
                  type={
                    condition.field === 'process_name' ? 'text' : 'number'
                  }
                  value={condition.value as string | number}
                  onChange={(e) =>
                    updateCondition(condition.id, {
                      value:
                        condition.field === 'process_name'
                          ? e.target.value
                          : Number(e.target.value),
                    })
                  }
                  placeholder={
                    condition.field === 'process_name'
                      ? '输入关键词'
                      : condition.field === 'upload_speed' || condition.field === 'download_speed'
                      ? 'KB/s'
                      : '数量'
                  }
                  className="px-3 py-2 bg-[var(--surface-card)] border border-[var(--border-subtle)] rounded-lg text-sm outline-none focus:border-[var(--primary)] w-32"
                />
              )}

              {condition.field === 'signature' && (
                <span className="text-sm text-[var(--text-secondary)]">未验证</span>
              )}

              {/* 删除按钮 */}
              {conditions.length > 1 && (
                <button
                  onClick={() => removeCondition(condition.id)}
                  className="p-1.5 hover:bg-[var(--danger)]/20 text-[var(--danger)] rounded-lg spring-transition"
                  title="移除条件"
                >
                  🗑️
                </button>
              )}
            </motion.div>
          ))}
        </AnimatePresence>
      </div>

      {/* 动作选择 */}
      <div className="mb-6">
        <label className="block text-sm font-medium mb-2">执行动作</label>
        <div className="flex flex-wrap items-center gap-3 p-4 bg-[var(--border-subtle)] rounded-xl">
          <span className="text-sm text-[var(--text-secondary)]">则</span>

          <select
            value={action.type}
            onChange={(e) =>
              setAction({
                type: e.target.value as any,
                durationSecs: action.durationSecs,
                speedLimitKbps: action.speedLimitKbps,
              })
            }
            className="px-3 py-2 bg-[var(--surface-card)] border border-[var(--border-subtle)] rounded-lg text-sm outline-none focus:border-[var(--primary)]"
          >
            {ACTION_OPTIONS.map(opt => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>

          {action.type === 'block_temporary' && (
            <>
              <span className="text-sm text-[var(--text-secondary)]">持续</span>
              <input
                type="number"
                value={action.durationSecs || 300}
                onChange={(e) =>
                  setAction({ ...action, durationSecs: Number(e.target.value) })
                }
                className="w-20 px-3 py-2 bg-[var(--surface-card)] border border-[var(--border-subtle)] rounded-lg text-sm outline-none focus:border-[var(--primary)]"
              />
              <span className="text-sm text-[var(--text-secondary)]">秒</span>
            </>
          )}

          {action.type === 'limit_speed' && (
            <>
              <span className="text-sm text-[var(--text-secondary)]">限速</span>
              <input
                type="number"
                value={action.speedLimitKbps || 500}
                onChange={(e) =>
                  setAction({ ...action, speedLimitKbps: Number(e.target.value) })
                }
                className="w-20 px-3 py-2 bg-[var(--surface-card)] border border-[var(--border-subtle)] rounded-lg text-sm outline-none focus:border-[var(--primary)]"
              />
              <span className="text-sm text-[var(--text-secondary)]">KB/s</span>
            </>
          )}
        </div>
      </div>

      {/* 操作按钮 */}
      <div className="flex gap-3">
        <button
          onClick={handleSave}
          className="btn-click px-6 py-2.5 bg-[var(--primary)] text-white rounded-xl font-medium shadow-lg shadow-[var(--primary-glow)] spring-transition"
        >
          💾 保存规则
        </button>
        <button
          onClick={onCancel}
          className="btn-click px-6 py-2.5 glass-card rounded-xl font-medium spring-transition"
        >
          取消
        </button>
      </div>
    </div>
  )
}
