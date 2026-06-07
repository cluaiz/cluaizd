/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
    "../../src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        cyber: {
          bg: '#020208',
          panel: '#0a0a16',
          border: '#1f1f3a',
          neonGreen: '#39FF14',
          neonBlue: '#00F0FF',
          neonPink: '#FF007F',
          neonYellow: '#FFEA00',
          text: '#a0a5c0',
          heading: '#ffffff',
        }
      },
      fontFamily: {
        mono: ['ui-monospace', 'Consolas', 'monospace'],
        sans: ['system-ui', 'Segoe UI', 'Roboto', 'sans-serif'],
      }
    },
  },
  plugins: [],
}
