import React from 'react'
import './DwightPersonaPanel.css'

const DwightPersonaPanel = () => {
  return (
    <div className="dwight-persona-panel">
      <div className="gradient-overlay">
        <img 
          src="/assets/soft-gradient.svg" 
          alt="Soft gradient overlay" 
          className="gradient-background"
        />
      </div>
      
      <div className="panel-content">
        <div className="dwight-header">
          <h2>Dwight AI</h2>
          <p>Speech Visualization</p>
        </div>
        
        <div className="circular-waveform-container">
          <img 
            src="/assets/circular-waveform.svg" 
            alt="Circular waveform visualization" 
            className="circular-waveform"
          />
        </div>
        
        <div className="status-info">
          <div className="status-item">
            <span className="status-label">Status:</span>
            <span className="status-value active">Active</span>
          </div>
          <div className="status-item">
            <span className="status-label">Mode:</span>
            <span className="status-value">AI Speech</span>
          </div>
          <div className="status-item">
            <span className="status-label">Level:</span>
            <span className="status-value">85 dB</span>
          </div>
        </div>
      </div>
    </div>
  )
}

export default DwightPersonaPanel