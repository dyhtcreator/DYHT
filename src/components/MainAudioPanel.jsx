import React from 'react'
import './MainAudioPanel.css'

const MainAudioPanel = () => {
  return (
    <div className="main-audio-panel">
      <div className="gradient-overlay">
        <img 
          src="/assets/soft-gradient.svg" 
          alt="Soft gradient overlay" 
          className="gradient-background"
        />
      </div>
      
      <div className="panel-content">
        <div className="panel-header">
          <h1>Audio Clip Inspection</h1>
          <div className="controls">
            <button className="control-btn active">Play</button>
            <button className="control-btn">Pause</button>
            <button className="control-btn">Stop</button>
          </div>
        </div>
        
        <div className="waveform-section">
          <div className="waveform-header">
            <h3>Frequency Analysis</h3>
            <div className="waveform-controls">
              <select className="frequency-selector">
                <option>Full Spectrum</option>
                <option>Low Frequency</option>
                <option>Mid Frequency</option>
                <option>High Frequency</option>
              </select>
              <button className="zoom-btn">Zoom</button>
            </div>
          </div>
          
          <div className="linear-waveform-container">
            <img 
              src="/assets/linear-waveform.svg" 
              alt="Linear waveform visualization" 
              className="linear-waveform"
            />
          </div>
        </div>
        
        <div className="analysis-info">
          <div className="info-row">
            <div className="info-item">
              <span className="info-label">Duration:</span>
              <span className="info-value">2:34.567</span>
            </div>
            <div className="info-item">
              <span className="info-label">Sample Rate:</span>
              <span className="info-value">48 kHz</span>
            </div>
            <div className="info-item">
              <span className="info-label">Bit Depth:</span>
              <span className="info-value">24-bit</span>
            </div>
            <div className="info-item">
              <span className="info-label">Peak Level:</span>
              <span className="info-value">-3.2 dB</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default MainAudioPanel