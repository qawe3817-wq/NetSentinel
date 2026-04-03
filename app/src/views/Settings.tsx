import { useState } from 'react'

export default function Settings() {
  const [activeTab, setActiveTab] = useState('general')

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">系统设置</h2>

      <div className="flex gap-6">
        {/* Settings Tabs */}
        <div className="glass-card rounded-2xl p-4 w-64 h-fit">
          {[
            { id: 'general', label: '通用设置', icon: '⚙️' },
            { id: 'startup', label: '启动项管理', icon: '🚀' },
            { id: 'network', label: '网络适配器', icon: '🌐' },
            { id: 'appearance', label: '外观定制', icon: '🎨' },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg text-left spring-transition ${
                activeTab === tab.id
                  ? 'bg-[var(--primary-glow)] text-[var(--primary)]'
                  : 'hover:bg-[var(--border-subtle)]'
              }`}
            >
              <span className="text-xl">{tab.icon}</span>
              <span className="font-medium">{tab.label}</span>
            </button>
          ))}
        </div>

        {/* Settings Content */}
        <div className="flex-1 glass-card rounded-2xl p-6">
          {activeTab === 'general' && (
            <div className="space-y-6">
              <h3 className="text-lg font-semibold">通用设置</h3>
              
              <div className="space-y-4">
                <div className="flex items-center justify-between p-4 bg-[var(--border-subtle)] rounded-xl">
                  <div>
                    <h4 className="font-medium">开机自启</h4>
                    <p className="text-sm text-[var(--text-secondary)]">系统启动时自动运行 NetSentinel</p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox" defaultChecked className="sr-only peer" />
                    <div className="w-11 h-6 bg-gray-300 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--primary)]"></div>
                  </label>
                </div>

                <div className="flex items-center justify-between p-4 bg-[var(--border-subtle)] rounded-xl">
                  <div>
                    <h4 className="font-medium">静默模式</h4>
                    <p className="text-sm text-[var(--text-secondary)]">阻断时不显示通知弹窗</p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox" className="sr-only peer" />
                    <div className="w-11 h-6 bg-gray-300 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--primary)]"></div>
                  </label>
                </div>

                <div className="flex items-center justify-between p-4 bg-[var(--border-subtle)] rounded-xl">
                  <div>
                    <h4 className="font-medium">看门狗自愈</h4>
                    <p className="text-sm text-[var(--text-secondary)]">核心服务崩溃后自动重启</p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox" defaultChecked className="sr-only peer" />
                    <div className="w-11 h-6 bg-gray-300 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--primary)]"></div>
                  </label>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'startup' && (
            <div className="space-y-6">
              <h3 className="text-lg font-semibold">启动项管理</h3>
              
              <div className="space-y-3">
                {[
                  { name: 'NetSentinel Core', publisher: 'NetSentinel Team', verified: true },
                  { name: 'Video Client', publisher: 'Unknown', verified: false },
                  { name: 'Cloud Drive', publisher: 'Tech Corp', verified: true },
                ].map((item, i) => (
                  <div
                    key={i}
                    className="p-4 bg-[var(--border-subtle)] rounded-xl flex items-center justify-between"
                  >
                    <div className="flex items-center gap-3">
                      <span className={item.verified ? 'text-green-500' : 'text-yellow-500'}>
                        {item.verified ? '✓' : '⚠'}
                      </span>
                      <div>
                        <h4 className="font-medium">{item.name}</h4>
                        <p className="text-xs text-[var(--text-secondary)]">
                          {item.publisher} {item.verified && '· 已签名'}
                        </p>
                      </div>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" defaultChecked={i === 0} className="sr-only peer" />
                      <div className="w-11 h-6 bg-gray-300 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--primary)]"></div>
                    </label>
                  </div>
                ))}
              </div>
            </div>
          )}

          {activeTab === 'network' && (
            <div className="space-y-6">
              <h3 className="text-lg font-semibold">网络适配器</h3>
              
              <div className="space-y-3">
                {[
                  { name: 'Wi-Fi (Intel AX200)', status: '活跃', type: '无线' },
                  { name: 'Ethernet (Realtek PCIe)', status: '已断开', type: '有线' },
                  { name: 'VMware Network Adapter', status: '活跃', type: '虚拟' },
                ].map((adapter, i) => (
                  <div
                    key={i}
                    className="p-4 bg-[var(--border-subtle)] rounded-xl flex items-center justify-between"
                  >
                    <div>
                      <h4 className="font-medium">{adapter.name}</h4>
                      <p className="text-xs text-[var(--text-secondary)]">
                        {adapter.type} · {adapter.status}
                      </p>
                    </div>
                    <button className="btn-click px-4 py-2 glass-card rounded-lg text-sm">
                      配置策略
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}

          {activeTab === 'appearance' && (
            <div className="space-y-6">
              <h3 className="text-lg font-semibold">外观定制</h3>
              
              <div className="grid grid-cols-3 gap-4">
                {[
                  { name: '极客黑', colors: ['#0F1115', '#1E2028', '#60A5FA'] },
                  { name: '纯净白', colors: ['#F0F2F5', '#FFFFFF', '#3B82F6'] },
                  { name: '赛博朋克', colors: ['#0D0D1A', '#1A1A2E', '#F0ABFC'] },
                ].map((theme, i) => (
                  <button
                    key={i}
                    className="p-4 rounded-xl border-2 border-[var(--border-subtle)] hover:border-[var(--primary)] spring-transition"
                  >
                    <div className="flex gap-1 mb-3">
                      {theme.colors.map((color, j) => (
                        <div
                          key={j}
                          className="w-8 h-8 rounded-lg"
                          style={{ backgroundColor: color }}
                        ></div>
                      ))}
                    </div>
                    <span className="text-sm font-medium">{theme.name}</span>
                  </button>
                ))}
              </div>

              <div className="pt-4">
                <h4 className="font-medium mb-3">自定义主色</h4>
                <input
                  type="color"
                  defaultValue="#3B82F6"
                  className="w-full h-12 rounded-lg cursor-pointer"
                />
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
