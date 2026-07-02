import React from 'react'
import ReactDOM from 'react-dom/client'
import { invoke } from '@tauri-apps/api/core'
import App from './App'
import './index.css'

function installClientErrorLogger() {
  const report = (kind: string, message: string) => {
    invoke('record_client_error', { kind, message }).catch(() => {})
  }

  window.addEventListener('error', (event) => {
    const location = event.filename
      ? `${event.filename}:${event.lineno}:${event.colno}`
      : 'unknown'
    report('error', `${event.message} @ ${location}`)
  })

  window.addEventListener('unhandledrejection', (event) => {
    report('unhandledrejection', String(event.reason))
  })
}

installClientErrorLogger()

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
