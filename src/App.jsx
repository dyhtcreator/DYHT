import React from 'react'
import BackgroundLayer from './components/BackgroundLayer'
import DwightPersonaPanel from './components/DwightPersonaPanel'
import MainAudioPanel from './components/MainAudioPanel'
import './App.css'

function App() {
  return (
    <div className="app">
      <BackgroundLayer />
      <div className="app-content">
        <div className="left-panel">
          <DwightPersonaPanel />
        </div>
        <div className="main-panel">
          <MainAudioPanel />
        </div>
      </div>
    </div>
  )
}

export default App