import { invoke } from '@tauri-apps/api/tauri';
import './style.css';

// Type definitions
interface AudioClip {
  id: string;
  timestamp: string;
  duration_ms: number;
  file_path: string;
  transcription?: string;
  waveform_data?: number[];
  tags: string[];
}

interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
}

// Application state
class DyhtApp {
  private isRecording = false;
  private audioContext: AudioContext | null = null;
  private liveWaveformCanvas: HTMLCanvasElement | null = null;
  private playbackWaveformCanvas: HTMLCanvasElement | null = null;
  private chatHistory: ChatMessage[] = [];
  private audioClips: AudioClip[] = [];
  private isAdminVerified = false;

  constructor() {
    this.init();
  }

  async init() {
    await this.setupEventListeners();
    await this.initializeAudio();
    await this.loadInitialData();
    this.setupWaveformCanvases();
    this.startLiveWaveform();
  }

  private async setupEventListeners() {
    // Chat functionality
    const chatInput = document.getElementById('chat-input') as HTMLInputElement;
    const sendBtn = document.getElementById('send-btn') as HTMLButtonElement;
    
    sendBtn?.addEventListener('click', () => this.sendMessage());
    chatInput?.addEventListener('keypress', (e) => {
      if (e.key === 'Enter') this.sendMessage();
    });

    // Audio controls
    const recordBtn = document.getElementById('record-btn') as HTMLButtonElement;
    const stopBtn = document.getElementById('stop-btn') as HTMLButtonElement;
    
    recordBtn?.addEventListener('click', () => this.toggleRecording());
    stopBtn?.addEventListener('click', () => this.stopRecording());

    // Settings
    const adminCodeInput = document.getElementById('admin-code') as HTMLInputElement;
    const verifyAdminBtn = document.getElementById('verify-admin') as HTMLButtonElement;
    const modelSelect = document.getElementById('model-select') as HTMLSelectElement;
    const themeSelect = document.getElementById('theme-select') as HTMLSelectElement;

    verifyAdminBtn?.addEventListener('click', () => this.verifyAdminCode(adminCodeInput.value));
    modelSelect?.addEventListener('change', (e) => this.switchModel((e.target as HTMLSelectElement).value));
    themeSelect?.addEventListener('change', (e) => this.switchTheme((e.target as HTMLSelectElement).value));

    // Emergency controls
    const killSwitch = document.getElementById('kill-switch') as HTMLButtonElement;
    killSwitch?.addEventListener('click', () => this.emergencyKillSwitch());

    // Panel controls
    const minimizeBtn = document.getElementById('minimize-btn') as HTMLButtonElement;
    const closeBtn = document.getElementById('close-btn') as HTMLButtonElement;
    
    minimizeBtn?.addEventListener('click', () => this.minimizePanel());
    closeBtn?.addEventListener('click', () => this.closePanel());

    // Trigger checkboxes
    const voiceActivation = document.getElementById('voice-activation') as HTMLInputElement;
    const autoTranscribe = document.getElementById('auto-transcribe') as HTMLInputElement;
    const smartResponse = document.getElementById('smart-response') as HTMLInputElement;

    voiceActivation?.addEventListener('change', (e) => this.toggleVoiceActivation((e.target as HTMLInputElement).checked));
    autoTranscribe?.addEventListener('change', (e) => this.toggleAutoTranscribe((e.target as HTMLInputElement).checked));
    smartResponse?.addEventListener('change', (e) => this.toggleSmartResponse((e.target as HTMLInputElement).checked));
  }

  private async initializeAudio() {
    try {
      this.audioContext = new AudioContext();
      console.log('Audio context initialized');
      console.log('Audio context state:', this.audioContext.state);
    } catch (error) {
      console.error('Failed to initialize audio context:', error);
      this.audioContext = null;
    }
  }

  private setupWaveformCanvases() {
    this.liveWaveformCanvas = document.getElementById('live-waveform') as HTMLCanvasElement;
    this.playbackWaveformCanvas = document.getElementById('playback-waveform') as HTMLCanvasElement;
    
    // Initialize playback waveform with placeholder data
    if (this.playbackWaveformCanvas) {
      const ctx = this.playbackWaveformCanvas.getContext('2d');
      if (ctx) {
        ctx.fillStyle = '#333333';
        ctx.fillRect(0, 0, this.playbackWaveformCanvas.width, this.playbackWaveformCanvas.height);
        ctx.fillStyle = '#00ff00';
        ctx.fillText('Playback waveform will appear here', 10, 50);
      }
    }
  }

  private startLiveWaveform() {
    if (!this.liveWaveformCanvas) return;

    const canvas = this.liveWaveformCanvas;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const drawCircularWaveform = () => {
      const centerX = canvas.width / 2;
      const centerY = canvas.height / 2;
      const radius = 80;

      ctx.clearRect(0, 0, canvas.width, canvas.height);
      
      // Draw background circle
      ctx.strokeStyle = '#333333';
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(centerX, centerY, radius, 0, 2 * Math.PI);
      ctx.stroke();

      // Draw waveform
      ctx.strokeStyle = '#00ff00';
      ctx.lineWidth = 3;
      ctx.beginPath();

      const time = Date.now() * 0.001;
      for (let i = 0; i < 360; i += 5) {
        const angle = (i * Math.PI) / 180;
        const amplitude = this.isRecording ? 
          Math.sin(time * 3 + i * 0.1) * 20 + Math.sin(time * 7 + i * 0.05) * 10 :
          Math.sin(time + i * 0.02) * 5;
        
        const x = centerX + Math.cos(angle) * (radius + amplitude);
        const y = centerY + Math.sin(angle) * (radius + amplitude);
        
        if (i === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
      }
      ctx.closePath();
      ctx.stroke();

      requestAnimationFrame(drawCircularWaveform);
    };

    drawCircularWaveform();
  }

  private async loadInitialData() {
    try {
      await invoke('init_agent');
      this.addChatMessage('assistant', 'Dwight AI agent initialized. Ready to assist!');
    } catch (error) {
      console.error('Failed to initialize agent:', error);
      this.addChatMessage('assistant', 'Failed to initialize agent. Please check the console for errors.');
    }
  }

  private async sendMessage() {
    const chatInput = document.getElementById('chat-input') as HTMLInputElement;
    const message = chatInput.value.trim();
    
    if (!message) return;

    this.addChatMessage('user', message);
    chatInput.value = '';

    try {
      const response = await invoke('chat_with_agent', { message }) as string;
      this.addChatMessage('assistant', response);
    } catch (error) {
      console.error('Chat error:', error);
      this.addChatMessage('assistant', 'Sorry, I encountered an error processing your message.');
    }
  }

  private addChatMessage(role: 'user' | 'assistant', content: string) {
    const message: ChatMessage = {
      role,
      content,
      timestamp: new Date().toISOString()
    };

    this.chatHistory.push(message);

    const chatMessages = document.getElementById('chat-messages');
    if (chatMessages) {
      const messageElement = document.createElement('div');
      messageElement.className = `chat-message ${role}`;
      messageElement.textContent = content;
      chatMessages.appendChild(messageElement);
      chatMessages.scrollTop = chatMessages.scrollHeight;
    }
  }

  private async toggleRecording() {
    if (!this.isRecording) {
      await this.startRecording();
    } else {
      await this.stopRecording();
    }
  }

  private async startRecording() {
    try {
      this.isRecording = true;
      const recordBtn = document.getElementById('record-btn') as HTMLButtonElement;
      const stopBtn = document.getElementById('stop-btn') as HTMLButtonElement;
      const statusText = document.getElementById('recording-status') as HTMLSpanElement;

      recordBtn.disabled = true;
      stopBtn.disabled = false;
      statusText.textContent = 'Recording...';
      statusText.style.color = '#ff3333';

      // TODO: Implement actual audio recording
      console.log('Recording started (placeholder)');

    } catch (error) {
      console.error('Recording error:', error);
      this.isRecording = false;
    }
  }

  private async stopRecording() {
    try {
      this.isRecording = false;
      const recordBtn = document.getElementById('record-btn') as HTMLButtonElement;
      const stopBtn = document.getElementById('stop-btn') as HTMLButtonElement;
      const statusText = document.getElementById('recording-status') as HTMLSpanElement;

      recordBtn.disabled = false;
      stopBtn.disabled = true;
      statusText.textContent = 'Processing...';
      statusText.style.color = '#ff6b35';

      // TODO: Process recorded audio
      const audioData = new Uint8Array([1, 2, 3, 4, 5]); // Placeholder
      const transcription = await invoke('process_audio', { audioData: Array.from(audioData) }) as string;
      
      statusText.textContent = 'Ready';
      statusText.style.color = '#cccccc';

      this.addChatMessage('assistant', `Audio processed: ${transcription}`);
      this.updateAudioClipsList();

    } catch (error) {
      console.error('Stop recording error:', error);
    }
  }

  private updateAudioClipsList() {
    const clipsList = document.getElementById('clips-list');
    if (!clipsList) return;

    // TODO: Get actual clips from backend
    clipsList.innerHTML = `
      <div class="clip-item">
        <div class="clip-info">
          <div class="clip-name">Recording ${new Date().toLocaleTimeString()}</div>
          <div class="clip-meta">5.2s • Just now</div>
        </div>
        <div class="clip-actions">
          <button class="clip-btn play">▶</button>
          <button class="clip-btn delete">🗑</button>
        </div>
      </div>
    `;
    
    // Update the audio clips array for future use
    this.audioClips.push({
      id: `clip-${Date.now()}`,
      timestamp: new Date().toISOString(),
      duration_ms: 5200,
      file_path: 'placeholder.wav',
      transcription: 'Placeholder transcription',
      waveform_data: [],
      tags: ['recorded']
    });
  }

  private async verifyAdminCode(adminCode: string) {
    if (!adminCode) {
      alert('Please enter an admin code');
      return;
    }

    try {
      // TODO: Implement actual admin verification
      this.isAdminVerified = true;
      const adminCodeInput = document.getElementById('admin-code') as HTMLInputElement;
      if (adminCodeInput) {
        adminCodeInput.style.borderColor = '#00ff00';
      }
      alert('Admin access granted');
    } catch (error) {
      console.error('Admin verification error:', error);
      const adminCodeInput = document.getElementById('admin-code') as HTMLInputElement;
      if (adminCodeInput) {
        adminCodeInput.style.borderColor = '#ff3333';
      }
      alert('Invalid admin code');
    }
  }

  private async switchModel(model: string) {
    try {
      console.log(`Switching to model: ${model}`);
      this.addChatMessage('assistant', `Switched to ${model} model`);
    } catch (error) {
      console.error('Model switch error:', error);
    }
  }

  private switchTheme(theme: string) {
    // TODO: Implement theme switching
    console.log(`Switching to theme: ${theme}`);
    console.log('Admin verified:', this.isAdminVerified);
  }

  private async emergencyKillSwitch() {
    const confirmed = confirm('Are you sure you want to activate the emergency kill switch? This will shut down all AI agent functions.');
    
    if (confirmed) {
      try {
        await invoke('emergency_kill_switch');
        this.addChatMessage('assistant', 'Emergency shutdown activated. All AI functions disabled.');
        
        // Disable all interactive elements
        const buttons = document.querySelectorAll('button:not(#kill-switch)');
        buttons.forEach(btn => (btn as HTMLButtonElement).disabled = true);
        
      } catch (error) {
        console.error('Emergency shutdown error:', error);
      }
    }
  }

  private minimizePanel() {
    const panel = document.getElementById('dwight-panel');
    if (panel) {
      panel.style.height = '40px';
      panel.querySelector('.panel-content')?.setAttribute('style', 'display: none');
    }
  }

  private closePanel() {
    const panel = document.getElementById('dwight-panel');
    if (panel) {
      panel.style.display = 'none';
    }
  }

  private toggleVoiceActivation(enabled: boolean) {
    console.log(`Voice activation: ${enabled}`);
    // TODO: Implement voice activation
  }

  private toggleAutoTranscribe(enabled: boolean) {
    console.log(`Auto-transcribe: ${enabled}`);
    // TODO: Implement auto-transcribe
  }

  private toggleSmartResponse(enabled: boolean) {
    console.log(`Smart response: ${enabled}`);
    // TODO: Implement smart response
  }
}

// Initialize the application when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
  new DyhtApp();
});

// Export for potential use in other modules
export default DyhtApp;