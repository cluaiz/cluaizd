import React, { useEffect } from 'react';
import { Menu, X } from 'lucide-react';
import { useDbStore } from '../../store/useDbStore';
import type { AccentColorType } from '../../store/useDbStore';
import { PrimaryRail } from './PrimaryRail';
import { SecondarySidebar } from './SecondarySidebar';
import { SearchModal } from '../ui/SearchModal';

interface AppShellProps {
  children: React.ReactNode;
}

export function AppShell({ children }: AppShellProps) {
  const isSidebarOpen = useDbStore((state) => state.isSidebarOpen);
  const toggleSidebar = useDbStore((state) => state.toggleSidebar);
  const setSidebarOpen = useDbStore((state) => state.setSidebarOpen);
  const settings = useDbStore((state) => state.settings);

  // Desktop slide/visibility states
  const isSidebarVisible = useDbStore((state) => state.isSidebarVisible);
  const toggleSidebarVisible = useDbStore((state) => state.toggleSidebarVisible);
  const setSidebarVisible = useDbStore((state) => state.setSidebarVisible);
  const isSecondaryOpen = useDbStore((state) => state.isSecondaryOpen);
  const toggleSecondary = useDbStore((state) => state.toggleSecondary);

  // Track if the sidebar was opened via hovering the edge
  const isHoverOpenedRef = React.useRef(false);

  // 1. Keyboard Shortcut Listener (Ctrl + B / Ctrl + Shift + B)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && (e.key === 'b' || e.key === 'B')) {
        e.preventDefault();
        if (e.shiftKey) {
          toggleSecondary();
        } else {
          toggleSidebarVisible();
          isHoverOpenedRef.current = false; // Reset hover peeking state since it's manual
        }
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [toggleSidebarVisible, toggleSecondary]);

  // 2. Mouse Move Edge Detection (<= 10px to open, > width + 40px to hide)
  useEffect(() => {
    if (!settings.hoverSidebar) return;

    const handleMouseMove = (e: MouseEvent) => {
      const sidebarWidth = isSecondaryOpen ? 312 : 136;

      // Mouse enters left-edge peeking zone (<= 10px) -> Show sidebar
      if (e.clientX <= 10 && !isSidebarVisible) {
        setSidebarVisible(true);
        isHoverOpenedRef.current = true;
      }

      // Mouse leaves sidebar zone -> Hide sidebar ONLY if it was opened via hover peeking
      if (isSidebarVisible && isHoverOpenedRef.current && e.clientX > (sidebarWidth + 40)) {
        setSidebarVisible(false);
        isHoverOpenedRef.current = false;
      }
    };

    window.addEventListener('mousemove', handleMouseMove);
    return () => window.removeEventListener('mousemove', handleMouseMove);
  }, [settings.hoverSidebar, isSidebarVisible, isSecondaryOpen, setSidebarVisible]);

  // 3. Dynamic Google Fonts Loader
  useEffect(() => {
    const fontId = 'dynamic-google-fonts';
    let link = document.getElementById(fontId) as HTMLLinkElement;
    if (!link) {
      link = document.createElement('link');
      link.id = fontId;
      link.rel = 'stylesheet';
      document.head.appendChild(link);
      link.href = 'https://fonts.googleapis.com/css2?family=Inter:wght@300;400;700;900&family=Roboto:wght@300;400;700;900&family=Lato:wght@300;400;700;900&family=Montserrat:wght@300;400;700;900&family=Oswald:wght@300;400;700;900&family=Playfair+Display:wght@300;400;700;900&family=Merriweather:wght@300;400;700;900&family=Nunito:wght@300;400;700;900&family=Raleway:wght@300;400;700;900&family=Poppins:wght@300;400;700;900&family=Open+Sans:wght@300;400;700;900&family=Inconsolata:wght@300;400;700;900&display=swap';
    }
    
    // Apply font family directly to document body & overwrite tailwind's --font-sans variable globally
    if (settings.fontFamily) {
      document.body.style.fontFamily = `"${settings.fontFamily}", sans-serif`;
      document.documentElement.style.setProperty('--font-sans', `"${settings.fontFamily}", system-ui, sans-serif`);
    }
  }, [settings.fontFamily]);

  // 4. Dynamic Theme Color Config Variables Selector
  useEffect(() => {
    const accents = {
      blue: { hex: '#00F0FF', glow: 'rgba(0, 240, 255, 0.25)' },
      green: { hex: '#39FF14', glow: 'rgba(57, 255, 20, 0.25)' },
      pink: { hex: '#FF007F', glow: 'rgba(255, 0, 127, 0.25)' },
      yellow: { hex: '#FFEA00', glow: 'rgba(255, 234, 0, 0.25)' },
      purple: { hex: '#8B5CF6', glow: 'rgba(139, 92, 246, 0.25)' },
      orange: { hex: '#FF6B00', glow: 'rgba(255, 107, 0, 0.25)' },
      red: { hex: '#FF0000', glow: 'rgba(255, 0, 0, 0.25)' },
      teal: { hex: '#00F5D4', glow: 'rgba(0, 245, 212, 0.25)' }
    };
    const accent = accents[settings.accentColor as AccentColorType] || accents.blue;
    document.documentElement.style.setProperty('--cyber-accent', accent.hex);
    document.documentElement.style.setProperty('--cyber-accent-glow', accent.glow);
  }, [settings.accentColor]);

  // 5. Dynamic Dark/Light/OLED/Hacker Theme Variables Toggler
  useEffect(() => {
    const themes = {
      dark: {
        bg: '#020208',
        panel: '#0a0a16',
        border: '#1f1f3a',
        text: '#a0a5c0',
        heading: '#ffffff'
      },
      light: {
        bg: '#ffffff',
        panel: '#ffffff',
        border: '#000000',
        text: '#000000',
        heading: '#000000'
      },
      oled: {
        bg: '#000000',
        panel: '#080808',
        border: '#1a1a1a',
        text: '#8e92a6',
        heading: '#ffffff'
      },
      hacker: {
        bg: '#010201',
        panel: '#020502',
        border: '#051005',
        text: '#5cb85c',
        heading: '#39FF14'
      }
    };
    const t = themes[settings.theme] || themes.dark;
    document.documentElement.style.setProperty('--cyber-bg', t.bg);
    document.documentElement.style.setProperty('--cyber-panel', t.panel);
    document.documentElement.style.setProperty('--cyber-border', t.border);
    document.documentElement.style.setProperty('--cyber-text', t.text);
    document.documentElement.style.setProperty('--cyber-heading', t.heading);

    if (settings.theme === 'light') {
      document.documentElement.classList.add('light');
      document.documentElement.classList.remove('dark');
    } else {
      document.documentElement.classList.add('dark');
      document.documentElement.classList.remove('light');
    }
  }, [settings.theme]);

  // 6. Font Sizing scale
  useEffect(() => {
    document.documentElement.style.fontSize = `${settings.fontSize}px`;
  }, [settings.fontSize]);

  return (
    <div className="h-screen w-full flex flex-col bg-cyber-bg text-cyber-text overflow-hidden select-none" style={{ fontSize: '1rem' }}>
      
      {/* Mobile Top Header */}
      <header className="lg:hidden h-14 bg-cyber-panel/90 border-b-2 border-cyber-border flex items-center justify-between px-4 z-40">
        <div className="flex items-center gap-2">
          <div className="w-7 h-7 bg-cyber-neonGreen/10 border border-cyber-neonGreen flex items-center justify-center select-none">
            <span className="text-sm leading-none">🪼</span>
          </div>
          <span className="font-mono text-xs font-bold text-white uppercase tracking-wider">
            CLUAIZD Genome Canvas
          </span>
        </div>
        <button
          onClick={toggleSidebar}
          className="p-1.5 border-2 border-cyber-border bg-cyber-bg text-white hover:bg-white/5"
          style={{ borderRadius: 0 }}
        >
          {isSidebarOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
        </button>
      </header>

      <div className="flex flex-1 overflow-hidden relative">
        
        {/* Desktop Slideable Sidebar Wrapper */}
        <div 
          className="hidden lg:flex h-full flex-shrink-0 transition-all duration-300 ease-in-out overflow-hidden border-r-2 border-cyber-border/80"
          style={{ 
            width: isSidebarVisible ? (isSecondaryOpen ? '312px' : '136px') : '0px',
            borderRightWidth: isSidebarVisible ? '2px' : '0px'
          }}
        >
          <PrimaryRail />
          <SecondarySidebar />
        </div>

        {/* Mobile Sidebar Overlay Drawer */}
        {isSidebarOpen && (
          <div 
            className="lg:hidden fixed inset-0 z-50 bg-black/60 backdrop-blur-sm transition-opacity"
            onClick={() => setSidebarOpen(false)}
          />
        )}
        <div
          className={`lg:hidden fixed top-14 bottom-0 left-0 z-50 flex bg-cyber-bg border-r-2 border-cyber-border transition-transform duration-300 ${
            isSidebarOpen ? 'translate-x-0' : '-translate-x-full'
          }`}
        >
          <PrimaryRail />
          <SecondarySidebar />
        </div>

        {/* Main Workspace Frame */}
        <main className="flex-1 flex flex-col min-w-0 overflow-y-auto relative bg-cyber-bg p-6 gap-6">
          {children}
        </main>

      </div>
      <SearchModal />
    </div>
  );
}
