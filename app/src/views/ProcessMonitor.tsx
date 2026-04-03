export default function ProcessMonitor() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">进程监控</h2>
        
        {/* Smart Filters */}
        <div className="flex gap-2">
          {['高上传', '多连接', '未知签名', '白名单外'].map((filter) => (
            <button
              key={filter}
              className="btn-click px-4 py-2 glass-card rounded-lg text-sm hover:bg-[var(--primary-glow)] spring-transition"
            >
              {filter}
            </button>
          ))}
        </div>
      </div>

      {/* Super Grid - Virtual Scroll List */}
      <div className="glass-card rounded-2xl overflow-hidden">
        {/* Table Header */}
        <div className="grid grid-cols-12 gap-4 px-6 py-3 bg-[var(--border-subtle)] text-sm font-medium">
          <div className="col-span-3">进程名</div>
          <div className="col-span-2">上行速度</div>
          <div className="col-span-2">下行速度</div>
          <div className="col-span-2">连接数</div>
          <div className="col-span-2">状态</div>
          <div className="col-span-1">操作</div>
        </div>

        {/* Table Body - Placeholder for virtual scroll */}
        <div className="divide-y divide-[var(--border-subtle)]">
          {[
            { name: 'chrome.exe', upload: '1.2 MB/s', download: '5.8 MB/s', connections: 45, status: '正常' },
            { name: 'video_client.exe', upload: '8.5 MB/s', download: '0.3 MB/s', connections: 128, status: '可疑' },
            { name: 'svchost.exe', upload: '0.1 MB/s', download: '0.2 MB/s', connections: 12, status: '正常' },
            { name: 'unknown_proc.dll', upload: '15.2 MB/s', download: '0.1 MB/s', connections: 256, status: '阻断' },
            { name: 'steam.exe', upload: '0.5 MB/s', download: '12.3 MB/s', connections: 8, status: '正常' },
          ].map((proc, i) => (
            <div
              key={i}
              className="grid grid-cols-12 gap-4 px-6 py-3 hover:bg-[var(--primary-glow)] spring-transition cursor-pointer items-center"
            >
              <div className="col-span-3 flex items-center gap-3">
                <span className="w-8 h-8 rounded-lg bg-gradient-to-br from-blue-400 to-purple-500 flex items-center justify-center text-white text-xs font-bold">
                  {proc.name[0].toUpperCase()}
                </span>
                <span className="font-mono text-sm">{proc.name}</span>
              </div>
              <div className="col-span-2">
                <div className="flex items-center gap-2">
                  <div className="flex-1 h-4 bg-[var(--border-subtle)] rounded-full overflow-hidden">
                    <div
                      className="h-full bg-green-500 rounded-full"
                      style={{ width: `${Math.min(parseInt(proc.upload) * 5, 100)}%` }}
                    ></div>
                  </div>
                  <span className="text-xs w-16">{proc.upload}</span>
                </div>
              </div>
              <div className="col-span-2">
                <div className="flex items-center gap-2">
                  <div className="flex-1 h-4 bg-[var(--border-subtle)] rounded-full overflow-hidden">
                    <div
                      className="h-full bg-blue-500 rounded-full"
                      style={{ width: `${Math.min(parseInt(proc.download) * 2, 100)}%` }}
                    ></div>
                  </div>
                  <span className="text-xs w-16">{proc.download}</span>
                </div>
              </div>
              <div className="col-span-2 text-sm">{proc.connections}</div>
              <div className="col-span-2">
                <span
                  className={`px-2 py-1 rounded-full text-xs ${
                    proc.status === '正常'
                      ? 'bg-green-500/20 text-green-500'
                      : proc.status === '可疑'
                      ? 'bg-yellow-500/20 text-yellow-500'
                      : 'bg-red-500/20 text-red-500'
                  }`}
                >
                  {proc.status}
                </span>
              </div>
              <div className="col-span-1">
                <button className="p-1 hover:bg-[var(--border-subtle)] rounded spring-transition">
                  ⋮
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Floating Action Bar */}
      <div className="fixed bottom-6 left-1/2 transform -translate-x-1/2 glass-card rounded-xl px-6 py-3 flex gap-4 shadow-2xl">
        <button className="btn-click px-4 py-2 bg-[var(--danger)] text-white rounded-lg font-medium">
          🚫 批量阻断
        </button>
        <button className="btn-click px-4 py-2 glass-card rounded-lg font-medium">
          ✅ 加入白名单
        </button>
        <button className="btn-click px-4 py-2 glass-card rounded-lg font-medium">
          📁 在资源管理器中显示
        </button>
      </div>
    </div>
  )
}
