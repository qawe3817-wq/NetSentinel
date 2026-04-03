import { invoke } from '@tauri-apps/api/tauri';

/**
 * Process information structure matching Rust ProcessInfo
 */
export interface ProcessInfo {
  pid: number;
  name: string;
  path: string;
  upload_speed: number;
  download_speed: number;
  connection_count: number;
  is_signed: boolean;
  risk_score: number;
  parent_pid?: number;
}

/**
 * Traffic statistics matching Rust TrafficStats
 */
export interface TrafficStats {
  total_upload: number;
  total_download: number;
  upload_speed: number;
  download_speed: number;
  active_connections: number;
  blocked_connections: number;
}

/**
 * Rule condition matching Rust RuleCondition
 */
export interface RuleCondition {
  field: string;
  operator: string;
  value: string | number | boolean;
}

/**
 * Rule action types
 */
export type RuleAction =
  | { type: 'Block'; duration_secs?: number }
  | { type: 'Limit'; upload_kbps: number; download_kbps: number }
  | { type: 'Warn' }
  | { type: 'Allow' };

/**
 * Rule definition matching Rust Rule
 */
export interface Rule {
  id: string;
  name: string;
  enabled: boolean;
  conditions: RuleCondition[];
  action: RuleAction;
  priority: number;
}

/**
 * Threat event matching Rust ThreatEvent
 */
export interface ThreatEvent {
  timestamp: number;
  process_name: string;
  pid: number;
  target_ip: string;
  target_port: number;
  reason: string;
  action_taken: string;
}

/**
 * Protection modes matching Rust ProtectionMode
 */
export type ProtectionMode = 'Silent' | 'Blocking' | 'Passthrough';

/**
 * Core bridge API for IPC communication with Rust backend
 */
export const coreBridge = {
  /**
   * Initialize the bridge and attempt to connect to core service
   */
  async initialize(): Promise<void> {
    await invoke('initialize_bridge');
  },

  /**
   * Get all processes with network activity
   */
  async getProcesses(): Promise<ProcessInfo[]> {
    return invoke<ProcessInfo[]>('get_processes');
  },

  /**
   * Get real-time traffic statistics
   */
  async getTrafficStats(): Promise<TrafficStats> {
    return invoke<TrafficStats>('get_traffic_stats');
  },

  /**
   * Get all rules
   */
  async getRules(): Promise<Rule[]> {
    return invoke<Rule[]>('get_rules');
  },

  /**
   * Create or update a rule
   */
  async applyRule(rule: Rule): Promise<boolean> {
    return invoke<boolean>('apply_rule', { rule });
  },

  /**
   * Delete a rule
   */
  async deleteRule(ruleId: string): Promise<boolean> {
    return invoke<boolean>('delete_rule', { rule_id: ruleId });
  },

  /**
   * Get recent threat events
   */
  async getThreats(): Promise<ThreatEvent[]> {
    return invoke<ThreatEvent[]>('get_threats');
  },

  /**
   * Change protection mode
   */
  async setProtectionMode(mode: ProtectionMode): Promise<void> {
    return invoke('set_protection_mode', { mode });
  },

  /**
   * Get current protection mode
   */
  async getProtectionMode(): Promise<ProtectionMode> {
    return invoke<ProtectionMode>('get_protection_mode');
  },

  /**
   * Terminate a process
   */
  async terminateProcess(pid: number): Promise<boolean> {
    return invoke<boolean>('terminate_process', { pid });
  },

  /**
   * Add process to whitelist
   */
  async addToWhitelist(processPath: string): Promise<boolean> {
    return invoke<boolean>('add_to_whitelist', { process_path: processPath });
  },

  /**
   * Block a process temporarily
   */
  async blockProcess(pid: number, durationSecs: number): Promise<boolean> {
    return invoke<boolean>('block_process', { pid, duration_secs: durationSecs });
  },

  /**
   * Check core service connection status
   */
  async checkCoreConnection(): Promise<boolean> {
    return invoke<boolean>('check_core_connection');
  },

  /**
   * Refresh mock data (development only)
   */
  async refreshMockData(): Promise<void> {
    return invoke('refresh_mock_data');
  },
};

/**
 * Format bytes to human-readable string
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B/s';
  const k = 1024;
  const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * Format timestamp to readable date
 */
export function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

/**
 * Calculate risk level from score
 */
export function getRiskLevel(score: number): 'low' | 'medium' | 'high' | 'critical' {
  if (score < 0.3) return 'low';
  if (score < 0.6) return 'medium';
  if (score < 0.8) return 'high';
  return 'critical';
}
