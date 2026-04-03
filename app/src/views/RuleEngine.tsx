import { useState, useCallback } from 'react'
import { useRuleEngine } from '../hooks/useRuleEngine'
import RuleEditor, { type RuleDefinition } from '../components/RuleEditor'
import { motion, AnimatePresence } from 'framer-motion'

export default function RuleEngine() {
  const { rules, addRule, updateRule, deleteRule, toggleRule } = useRuleEngine()
  const [isEditing, setIsEditing] = useState(false)
  const [editingRule, setEditingRule] = useState<RuleDefinition | undefined>()

  const handleSaveRule = useCallback((rule: RuleDefinition) => {
    if (editingRule) {
      updateRule(rule)
    } else {
      addRule(rule)
    }
    setIsEditing(false)
    setEditingRule(undefined)
  }, [editingRule, addRule, updateRule])

  const handleCancelEdit = useCallback(() => {
    setIsEditing(false)
    setEditingRule(undefined)
  }, [])

  const handleEditRule = useCallback((rule: RuleDefinition) => {
    setEditingRule(rule)
    setIsEditing(true)
  }, [])

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">规则引擎</h2>
        <button
          onClick={() => {
            setEditingRule(undefined)
            setIsEditing(true)
          }}
          className="btn-click px-6 py-3 bg-[var(--primary)] text-white rounded-xl font-medium shadow-lg shadow-[var(--primary-glow)]"
        >
          + 新建规则
        </button>
      </div>

      <AnimatePresence>
        {isEditing && (
          <motion.div
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
            transition={{ type: 'spring', stiffness: 300, damping: 30 }}
          >
            <RuleEditor
              rule={editingRule}
              onSave={handleSaveRule}
              onCancel={handleCancelEdit}
            />
          </motion.div>
        )}
      </AnimatePresence>

      {/* Rule List */}
      <div className="glass-card rounded-2xl p-6">
        <h3 className="text-lg font-semibold mb-4">已启用规则</h3>
        
        <div className="space-y-3">
          {rules.map((rule) => (
            <motion.div
              key={rule.id}
              layout
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="p-4 rounded-xl bg-[var(--border-subtle)] flex items-center justify-between spring-transition hover:bg-[var(--primary-glow)] cursor-pointer"
              onClick={() => handleEditRule(rule)}
            >
              <div className="flex items-center gap-4">
                <input
                  type="checkbox"
                  checked={rule.enabled}
                  onChange={(e) => {
                    e.stopPropagation()
                    toggleRule(rule.id)
                  }}
                  className="w-5 h-5 rounded accent-[var(--primary)]"
                />
                <div>
                  <h4 className="font-medium">{rule.name}</h4>
                  <p className="text-sm text-[var(--text-secondary)]">
                    {rule.conditions.map(c => {
                      const fieldLabel = c.field === 'process_name' ? '进程名' :
                                        c.field === 'upload_speed' ? '上传速度' :
                                        c.field === 'download_speed' ? '下载速度' :
                                        c.field === 'connections' ? '连接数' : '签名状态'
                      const opLabel = c.operator === 'contains' ? '包含' :
                                     c.operator === 'equals' ? '等于' :
                                     c.operator === 'greater_than' ? '>' :
                                     c.operator === 'less_than' ? '<' : ''
                      const valueLabel = c.field === 'signature' ? '未验证' : 
                                       c.field.includes('speed') ? `${c.value} KB/s` :
                                       c.field === 'connections' ? `${c.value}` : `${c.value}`
                      return `${fieldLabel} ${opLabel} ${valueLabel}`
                    }).join(' 且 ')} → {
                      rule.action.type === 'block_temporary' ? `阻断 ${rule.action.durationSecs || 300}秒` :
                      rule.action.type === 'block_permanent' ? '永久阻断' :
                      rule.action.type === 'limit_speed' ? `限速 ${rule.action.speedLimitKbps || 500} KB/s` :
                      '仅警告'
                    }
                  </p>
                </div>
              </div>
              <div className="flex gap-2">
                <button
                  onClick={(e) => {
                    e.stopPropagation()
                    handleEditRule(rule)
                  }}
                  className="p-2 hover:bg-white/10 rounded-lg spring-transition"
                >
                  ✏️
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation()
                    deleteRule(rule.id)
                  }}
                  className="p-2 hover:bg-[var(--danger)]/20 rounded-lg spring-transition text-[var(--danger)]"
                >
                  🗑️
                </button>
              </div>
            </motion.div>
          ))}
          
          {rules.length === 0 && (
            <div className="text-center py-12 text-[var(--text-secondary)]">
              <p>暂无规则，点击右上角创建新规则</p>
            </div>
          )}
        </div>
      </div>

      {/* Cloud Sync */}
      <div className="glass-card rounded-2xl p-6">
        <h3 className="text-lg font-semibold mb-4">☁️ 社区规则</h3>
        <div className="text-sm text-[var(--text-secondary)] mb-4">
          导入来自社区的高质量规则，提升防护能力
        </div>
        
        <div className="space-y-3">
          {[
            { name: '主流 PCDN 特征库', author: '@NetSec Team', rating: 4.9, downloads: '12k' },
            { name: '游戏加速器白名单', author: '@GamerCN', rating: 4.7, downloads: '8.5k' },
            { name: '挖矿程序阻断列表', author: '@SecurityLab', rating: 4.8, downloads: '15k' },
          ].map((community, i) => (
            <div
              key={i}
              className="p-4 rounded-xl bg-[var(--border-subtle)] flex items-center justify-between"
            >
              <div>
                <h4 className="font-medium">{community.name}</h4>
                <p className="text-xs text-[var(--text-secondary)]">
                  {community.author} · ⭐ {community.rating} · ⬇️ {community.downloads}
                </p>
              </div>
              <button className="btn-click px-4 py-2 glass-card rounded-lg text-sm hover:bg-[var(--primary-glow)]">
                导入
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
