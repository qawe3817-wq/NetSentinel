import { BrowserRouter, Routes, Route } from 'react-router-dom'
import Layout from '@components/Layout'
import Dashboard from '@views/Dashboard'
import ProcessMonitor from '@views/ProcessMonitor'
import RuleEngine from '@views/RuleEngine'
import Settings from '@views/Settings'

function App() {
  return (
    <BrowserRouter>
      <Layout>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/monitor" element={<ProcessMonitor />} />
          <Route path="/rules" element={<RuleEngine />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  )
}

export default App
