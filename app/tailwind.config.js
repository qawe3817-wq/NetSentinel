/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Design Token - Background
        'bg-base': 'var(--bg-base)',
        'surface-card': 'var(--surface-card)',
        
        // Design Token - Primary
        'primary-glow': 'var(--primary-glow)',
        'primary': 'var(--primary)',
        
        // Design Token - Danger
        'danger-pulse': 'var(--danger-pulse)',
        'danger': 'var(--danger)',
        
        // Design Token - Text
        'text-main': 'var(--text-main)',
        'text-secondary': 'var(--text-secondary)',
        
        // Design Token - Border
        'border-subtle': 'var(--border-subtle)',
      },
      fontFamily: {
        sans: ['HarmonyOS Sans SC', 'Inter', 'Segoe UI Variable', 'sans-serif'],
      },
      backdropBlur: {
        'card': '20px',
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'bounce-spring': 'bounce 1s cubic-bezier(0.34, 1.56, 0.64, 1) infinite',
      },
    },
  },
  plugins: [],
}
