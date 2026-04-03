import { useState, useCallback, useEffect } from 'react'
import { coreBridge, type Rule as CoreRule, type RuleAction as CoreRuleAction } from '../../src-tauri/types'

export interface RuleCondition {
  field: 'processName' | 'uploadSpeed' | 'downloadSpeed' | 'connections' | 'signature'
  operator: 'contains' | 'equals' | 'greaterThan' | 'lessThan' | 'notVerified'
  value: string | number
}

export interface RuleAction {
  type: 'block' | 'limit' | 'warn' | 'allow'
  duration?: number // 分钟，0 表示永久
  limitSpeed?: number // KB/s
}

export interface Rule {
  id: string
  name: string
  conditions: RuleCondition[]
  action: RuleAction
  enabled: boolean
  order: number
  createdAt: number
  source?: 'local' | 'cloud'
  cloudAuthor?: string
  cloudRating?: number
}

export interface ConflictInfo {
  ruleId: string
  conflictType: 'overlap' | 'contradiction' | 'redundant'
  description: string
  suggestion: string
}

// Convert CoreRule to local Rule format
const convertCoreRule = (core: CoreRule): Rule => {
  const action: RuleAction = 
    core.action.type === 'Block' 
      ? { type: 'block', duration: core.action.duration_secs ? core.action.duration_secs / 60 : 0 }
      : core.action.type === 'Limit'
        ? { type: 'limit', limitSpeed: core.action.upload_kbps / 1024 }
        : core.action.type === 'Warn'
          ? { type: 'warn' }
          : { type: 'allow' }

  return {
    id: core.id,
    name: core.name,
    conditions: core.conditions.map(c => ({
      field: c.field as any,
      operator: c.operator as any,
      value: c.value,
    })),
    action,
    enabled: core.enabled,
    order: core.priority,
    createdAt: Date.now(),
    source: 'local' as const,
  }
}

// Convert local Rule to CoreRule format
const convertToCoreRule = (rule: Rule): CoreRule => {
  const action: CoreRuleAction =
    rule.action.type === 'block'
      ? { type: 'Block', duration_secs: rule.action.duration ? rule.action.duration * 60 : undefined }
      : rule.action.type === 'limit'
        ? { type: 'Limit', upload_kbps: (rule.action.limitSpeed || 0) * 1024, download_kbps: (rule.action.limitSpeed || 0) * 1024 }
        : rule.action.type === 'warn'
          ? { type: 'Warn' }
          : { type: 'Allow' }

  return {
    id: rule.id,
    name: rule.name,
    enabled: rule.enabled,
    conditions: rule.conditions.map(c => ({
      field: c.field,
      operator: c.operator,
      value: c.value,
    })),
    action,
    priority: rule.order,
  }
}

/**
 * 规则引擎 Hook
 * 支持可视化规则编辑、冲突检测、云同步
 */
export function useRuleEngine() {
  const [rules, setRules] = useState<Rule[]>([])
  const [loading, setLoading] = useState(true)
  const [editingRule, setEditingRule] = useState<Rule | null>(null)
  const [conflicts, setConflicts] = useState<ConflictInfo[]>([])

  // Load rules from core service
  useEffect(() => {
    const loadRules = async () => {
      try {
        const coreRules = await coreBridge.getRules()
        setRules(coreRules.map(convertCoreRule))
      } catch (err) {
        console.error('Failed to load rules:', err)
        // Use default rules on error
        setRules([getDefaultPCDNRule()])
      } finally {
        setLoading(false)
      }
    }
    loadRules()
  }, [])

  const getDefaultPCDNRule = (): Rule => ({
    id: `default-${Date.now()}`,
    name: 'PCDN 阻断规则',
    conditions: [
      { field: 'uploadSpeed', operator: 'greaterThan', value: 1024 },
      { field: 'connections', operator: 'greaterThan', value: 50 }
    ],
    action: { type: 'block', duration: 10 },
    enabled: true,
    order: 1,
    createdAt: Date.now(),
    source: 'local'
  })

  // 创建新规则
  const createRule = useCallback(async (rule: Omit<Rule, 'id' | 'createdAt'>) => {
    const newRule: Rule = {
      ...rule,
      id: `rule-${Date.now()}`,
      createdAt: Date.now()
    }
    
    // Sync with core service
    try {
      await coreBridge.applyRule(convertToCoreRule(newRule))
    } catch (err) {
      console.error('Failed to create rule in core:', err)
    }
    
    setRules(prev => [...prev, newRule].sort((a, b) => a.order - b.order))
    return newRule
  }, [])

  // 更新规则
  const updateRule = useCallback(async (ruleId: string, updates: Partial<Rule>) => {
    setRules(prev =>
      prev.map(r => (r.id === ruleId ? { ...r, ...updates } : r))
    )
    
    // Sync with core service
    const updatedRule = rules.find(r => r.id === ruleId)
    if (updatedRule) {
      try {
        await coreBridge.applyRule(convertToCoreRule({ ...updatedRule, ...updates }))
      } catch (err) {
        console.error('Failed to update rule in core:', err)
      }
    }
  }, [rules])

  // 删除规则
  const deleteRule = useCallback(async (ruleId: string) => {
    // Delete from core service first
    try {
      await coreBridge.deleteRule(ruleId)
    } catch (err) {
      console.error('Failed to delete rule from core:', err)
    }
    
    setRules(prev => prev.filter(r => r.id !== ruleId))
  }, [])

  // 切换规则启用状态
  const toggleRule = useCallback((ruleId: string) => {
    setRules(prev =>
      prev.map(r =>
        r.id === ruleId ? { ...r, enabled: !r.enabled } : r
      )
    )
  }, [])

  // 冲突检测
  const detectConflicts = useCallback((newRule?: Rule): ConflictInfo[] => {
    const rulesToCheck = newRule ? [...rules, newRule] : rules
    const detectedConflicts: ConflictInfo[] = []

    for (let i = 0; i < rulesToCheck.length; i++) {
      for (let j = i + 1; j < rulesToCheck.length; j++) {
        const ruleA = rulesToCheck[i]
        const ruleB = rulesToCheck[j]

        if (!ruleA.enabled || !ruleB.enabled) continue

        // 检查条件是否重叠
        const hasOverlap = ruleA.conditions.some(condA =>
          ruleB.conditions.some(condB =>
            condA.field === condB.field &&
            condA.operator === condB.operator
          )
        )

        if (hasOverlap) {
          // 检查动作是否矛盾
          if (
            (ruleA.action.type === 'block' && ruleB.action.type === 'allow') ||
            (ruleA.action.type === 'allow' && ruleB.action.type === 'block')
          ) {
            detectedConflicts.push({
              ruleId: ruleB.id,
              conflictType: 'contradiction',
              description: `规则 "${ruleA.name}" 和 "${ruleB.name}" 存在动作冲突`,
              suggestion: '建议调整其中一个规则的动作或禁用该规则'
            })
          } else if (
            ruleA.action.type === ruleB.action.type &&
            JSON.stringify(ruleA.action) === JSON.stringify(ruleB.action)
          ) {
            detectedConflicts.push({
              ruleId: ruleB.id,
              conflictType: 'redundant',
              description: `规则 "${ruleB.name}" 与 "${ruleA.name}" 功能重复`,
              suggestion: '考虑删除冗余规则以简化配置'
            })
          } else {
            detectedConflicts.push({
              ruleId: ruleB.id,
              conflictType: 'overlap',
              description: `规则 "${ruleA.name}" 和 "${ruleB.name}" 条件存在重叠`,
              suggestion: '建议调整条件范围以避免意外行为'
            })
          }
        }
      }
    }

    setConflicts(detectedConflicts)
    return detectedConflicts
  }, [rules])

  // 社区规则（云同步）
  const communityRules = useState<Rule[]>([
    {
      id: 'community-1',
      name: '主流 PCDN 特征库',
      conditions: [
        { field: 'uploadSpeed', operator: 'greaterThan', value: 2048 },
        { field: 'connections', operator: 'greaterThan', value: 100 }
      ],
      action: { type: 'block', duration: 30 },
      enabled: false,
      order: 100,
      createdAt: Date.now(),
      source: 'cloud',
      cloudAuthor: '@NetSec Team',
      cloudRating: 4.9
    },
    {
      id: 'community-2',
      name: '游戏加速器白名单',
      conditions: [
        { field: 'processName', operator: 'contains', value: 'accelerator' }
      ],
      action: { type: 'allow' },
      enabled: false,
      order: 101,
      createdAt: Date.now(),
      source: 'cloud',
      cloudAuthor: '@GamerCN',
      cloudRating: 4.7
    },
    {
      id: 'community-3',
      name: '挖矿程序阻断列表',
      conditions: [
        { field: 'processName', operator: 'contains', value: 'miner' },
        { field: 'signature', operator: 'notVerified', value: true }
      ],
      action: { type: 'block', duration: 0 },
      enabled: false,
      order: 102,
      createdAt: Date.now(),
      source: 'cloud',
      cloudAuthor: '@SecurityLab',
      cloudRating: 4.8
    }
  ])[0]

  // 导入社区规则
  const importCommunityRule = useCallback((ruleId: string) => {
    const ruleToImport = communityRules.find(r => r.id === ruleId)
    if (!ruleToImport) return

    const importedRule: Rule = {
      ...ruleToImport,
      id: `imported-${Date.now()}`,
      source: 'local',
      enabled: true,
      order: rules.length + 1
    }

    setRules(prev => [...prev, importedRule])
  }, [communityRules, rules.length])

  // 导出规则
  const exportRules = useCallback(() => {
    return JSON.stringify(rules, null, 2)
  }, [rules])

  // 导入规则
  const importRules = useCallback((jsonString: string) => {
    try {
      const importedRules = JSON.parse(jsonString) as Rule[]
      const maxOrder = Math.max(0, ...rules.map(r => r.order))
      
      const newRules = importedRules.map((rule, index) => ({
        ...rule,
        id: `imported-${Date.now()}-${index}`,
        order: maxOrder + index + 1,
        source: 'local' as const
      }))

      setRules(prev => [...prev, ...newRules])
      return { success: true, count: newRules.length }
    } catch (error) {
      return { success: false, error: '无效的规则格式' }
    }
  }, [rules])

  // 重置为默认规则
  const resetToDefaults = useCallback(() => {
    setRules([
      {
        id: 'default-1',
        name: 'PCDN 阻断规则',
        conditions: [
          { field: 'uploadSpeed', operator: 'greaterThan', value: 1024 },
          { field: 'connections', operator: 'greaterThan', value: 50 }
        ],
        action: { type: 'block', duration: 10 },
        enabled: true,
        order: 1,
        createdAt: Date.now(),
        source: 'local'
      }
    ])
    setConflicts([])
  }, [])

  return {
    rules,
    editingRule,
    setEditingRule,
    conflicts,
    createRule,
    updateRule,
    deleteRule,
    toggleRule,
    detectConflicts,
    communityRules,
    importCommunityRule,
    exportRules,
    importRules,
    resetToDefaults
  }
}
