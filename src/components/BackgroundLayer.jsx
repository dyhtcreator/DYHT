import React from 'react'
import './BackgroundLayer.css'

const BackgroundLayer = () => {
  return (
    <div className="background-layer">
      <div className="cloud-background">
        <img 
          src="/assets/cloud-background.svg" 
          alt="Cloud background" 
          className="cloud-image"
        />
      </div>
    </div>
  )
}

export default BackgroundLayer