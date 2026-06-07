import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import '../../../src/styles/index.css'
import App from '../../../src/App.tsx'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
