import { Sliders, Sun, Moon, Sparkles, Type, Palette, Terminal, Zap, Database } from 'lucide-react';
import { useDbStore } from '../../../store/useDbStore';
import type { AccentColorType } from '../../../store/useDbStore';

export function SettingsPanel() {
  const settings = useDbStore((state) => state.settings);
  const updateSettings = useDbStore((state) => state.updateSettings);
  const addLog = useDbStore((state) => state.addLog);
  const activeSettingsSubView = useDbStore((state) => state.activeSettingsSubView);
  const ttlValue = useDbStore((state) => state.ttlValue);
  const setTtlValue = useDbStore((state) => state.setTtlValue);

  const handleTtlChange = (value: number) => {
    setTtlValue(value);
    addLog('info', `DB state manager TTL threshold updated to ${value}s`);
  };

  const handleThemeChange = (theme: 'dark' | 'light' | 'oled' | 'hacker') => {
    updateSettings({ theme });
    addLog('success', `Aesthetic layout theme shifted to ${theme.toUpperCase()} mode`);
  };

  const handleFontSizeChange = (fontSize: number) => {
    updateSettings({ fontSize });
    addLog('info', `Active UI layout font scaling updated to ${fontSize}px`);
  };

  const handleFontFamilyChange = (fontFamily: string) => {
    updateSettings({ fontFamily });
    addLog('info', `Active UI typography font family updated to ${fontFamily}`);
  };

  const handleAccentColorChange = (accentColor: AccentColorType) => {
    updateSettings({ accentColor });
    addLog('success', `Accent theme color set to ${accentColor.toUpperCase()}`);
  };

  const handleHoverSidebarChange = (hoverSidebar: boolean) => {
    updateSettings({ hoverSidebar });
    addLog('info', `Edge-hover sidebar activation toggled ${hoverSidebar ? 'ON' : 'OFF'}`);
  };

  const fontOptions = [
    'Inter', 'Roboto', 'Lato', 'Montserrat',
    'Oswald', 'Playfair Display', 'Merriweather',
    'Nunito', 'Raleway', 'Poppins', 'Open Sans', 'Inconsolata'
  ];

  const colorOptions: { name: AccentColorType; color: string }[] = [
    { name: 'blue', color: '#0066FF' },   // Electric Cyber Blue (Glows on Dark, Solid on Light)
    { name: 'green', color: '#10B981' },  // Vibrant Emerald Green (High contrast everywhere)
    { name: 'pink', color: '#FF2A85' },   // Deep Neon Pink (Doesn't fade on white)
    { name: 'yellow', color: '#FFB800' }, // Cyber Gold/Amber (Highly readable on light backgrounds)
    { name: 'purple', color: '#7C3AED' }, // Vivid Royal Violet
    { name: 'orange', color: '#FF5500' }, // Blazing Safety Orange (Pops violently in both modes)
    { name: 'red', color: '#E11D48' },    // Neon Crimson Red
    { name: 'teal', color: '#00A896' }    // Deep Plasma Teal (Legible on light mode)
  ];

  const themeOptions = [
    { name: 'dark', label: 'DARK', icon: Moon },
    { name: 'light', label: 'LIGHT', icon: Sun },
    { name: 'oled', label: 'OLED BLACK', icon: Zap },
    { name: 'hacker', label: 'MATRIX', icon: Terminal }
  ] as const;

  return (
    <div className="w-full flex flex-col gap-6 animate-slideUp">
      
      {/* Page Header */}
      <div className="flex items-center gap-3 border-b-2 border-cyber-border pb-4">
        <div className="w-9 h-9 bg-cyber-accent/10 border-2 border-cyber-accent flex items-center justify-center shadow-[0px_0px_15px_var(--cyber-accent-glow)]">
          <Sliders className="w-4 h-4 text-cyber-accent" />
        </div>
        <div>
          <h2 className="text-lg font-black text-cyber-heading uppercase tracking-wider">
            Kernel Console Settings
          </h2>
          <p className="text-[10px] font-sans text-cyber-text/50 uppercase tracking-widest">
            Configure typography matrices, active color templates, and system themes
          </p>
        </div>
      </div>

      {/* Settings Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        
        {/* Card 1: Theme Color Palette */}
        <div className="bg-cyber-panel border-2 border-cyber-border p-5 shadow-[4px_4px_0px_0px_#000]">
          <div className="flex items-center gap-2 mb-4 border-b border-cyber-border/40 pb-2">
            <Palette className="w-4 h-4 text-cyber-accent" />
            <h3 className="text-xs font-mono font-bold text-cyber-heading uppercase tracking-wider">
              Aesthetic Theme Palette
            </h3>
          </div>
          
          <div className="flex flex-col gap-3">
            <label className="text-[10px] font-mono uppercase text-cyber-text/60">
              Active Color Palette Selector
            </label>
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
              {colorOptions.map((opt) => (
                <button
                  key={opt.name}
                  onClick={() => handleAccentColorChange(opt.name)}
                  className={`flex items-center justify-center gap-2 py-2 text-[10px] font-mono border-2 transition-all cursor-pointer ${settings.accentColor === opt.name
                      ? 'bg-cyber-accent border-cyber-border text-black font-bold shadow-[2px_2px_0px_0px_var(--cyber-accent-glow)]'
                      : 'bg-cyber-bg border-cyber-border text-cyber-text hover:bg-white/5'
                    }`}
                  style={{ borderRadius: 0 }}
                >
                  <span className="w-2.5 h-2.5 border border-white/20" style={{ backgroundColor: opt.color }} />
                  <span className="uppercase">{opt.name}</span>
                </button>
              ))}
            </div>
            <p className="text-[9px] font-sans text-cyber-text/40">
              Controls the primary interactive border highlights, headers, and UI glow colors globally.
            </p>
          </div>
        </div>

        {/* Card 2: Visual Environment Theme */}
        <div className="bg-cyber-panel border-2 border-cyber-border p-5 shadow-[4px_4px_0px_0px_#000]">
          <div className="flex items-center gap-2 mb-4 border-b border-cyber-border/40 pb-2">
            <Sparkles className="w-4 h-4 text-cyber-accent" />
            <h3 className="text-xs font-mono font-bold text-cyber-heading uppercase tracking-wider">
              Visual Environment
            </h3>
          </div>

          <div className="flex flex-col gap-3">
            <label className="text-[10px] font-mono uppercase text-cyber-text/60">
              Aesthetic Theme Mode
            </label>
            <div className="grid grid-cols-2 gap-3">
              {themeOptions.map((opt) => {
                const IconComponent = opt.icon;
                return (
                  <button
                    key={opt.name}
                    onClick={() => handleThemeChange(opt.name)}
                    className={`flex items-center justify-center gap-2 py-2.5 text-xs font-mono border-2 transition-all cursor-pointer ${settings.theme === opt.name
                        ? 'bg-cyber-accent border-cyber-border text-black font-bold shadow-[2px_2px_0px_0px_var(--cyber-accent-glow)]'
                        : 'bg-cyber-bg border-cyber-border text-cyber-text hover:bg-white/5'
                      }`}
                    style={{ borderRadius: 0 }}
                  >
                    <IconComponent className="w-3.5 h-3.5" />
                    <span>{opt.label}</span>
                  </button>
                );
              })}
            </div>
          </div>
        </div>

        {/* Card 3: Database Cache Configuration */}
        <div className="bg-cyber-panel border-2 border-cyber-border p-5 shadow-[4px_4px_0px_0px_#000]">
          <div className="flex items-center gap-2 mb-4 border-b border-cyber-border/40 pb-2">
            <Database className="w-4 h-4 text-cyber-accent" />
            <h3 className="text-xs font-mono font-bold text-cyber-heading uppercase tracking-wider">
              Database Cache Configuration
            </h3>
          </div>
          
          <div className="flex flex-col gap-3">
            <label className="text-[10px] font-mono uppercase text-cyber-text/60">
              Active TTL Threshold (Seconds)
            </label>
            <div className="flex items-center gap-2">
              <input
                type="number"
                min="30"
                max="3600"
                value={ttlValue}
                onChange={(e) => handleTtlChange(parseInt(e.target.value) || 300)}
                className="flex-1 bg-cyber-bg border-2 border-cyber-border px-3 py-2 text-xs font-mono text-cyber-heading outline-none focus:border-cyber-accent transition-all"
                style={{ borderRadius: 0 }}
              />
              <span className="text-[10px] font-mono text-cyber-text/40">SEC</span>
            </div>
            <p className="text-[9px] font-sans text-cyber-text/40">
              Sets the invalidation timer for mapped memory page registers in LMDB and SQLite.
            </p>
          </div>
        </div>

        {/* Card 4: Hover Sidebar Toggle */}
        <div className="bg-cyber-panel border-2 border-cyber-border p-5 shadow-[4px_4px_0px_0px_#000]">
          <h3 className="text-xs font-mono font-bold text-cyber-heading uppercase tracking-wider mb-4 border-b border-cyber-border/40 pb-2">
            Edge-Hover Sidebar Peeking
          </h3>

          <div className="grid grid-cols-2 gap-3">
            <button
              onClick={() => handleHoverSidebarChange(true)}
              className={`py-2 text-[10px] font-mono border-2 uppercase transition-all cursor-pointer ${settings.hoverSidebar === true
                  ? 'bg-cyber-accent border-cyber-border text-black font-bold shadow-[2px_2px_0px_0px_var(--cyber-accent-glow)]'
                  : 'bg-cyber-bg border-cyber-border text-cyber-text hover:bg-white/5'
                }`}
              style={{ borderRadius: 0 }}
            >
              ACTIVE (ON)
            </button>
            <button
              onClick={() => handleHoverSidebarChange(false)}
              className={`py-2 text-[10px] font-mono border-2 uppercase transition-all cursor-pointer ${settings.hoverSidebar === false
                  ? 'bg-cyber-accent border-cyber-border text-black font-bold shadow-[2px_2px_0px_0px_var(--cyber-accent-glow)]'
                  : 'bg-cyber-bg border-cyber-border text-cyber-text hover:bg-white/5'
                }`}
              style={{ borderRadius: 0 }}
            >
              INACTIVE (OFF)
            </button>
          </div>
          <p className="text-[9px] font-sans text-cyber-text/40 mt-3">
            When active, hovering your cursor within 10px of the left edge slides open the sidebar. Moving it away collapses it. Use Ctrl + B to pin/unpin it at any time.
          </p>
        </div>

        {/* Card 5: Dynamic Typography Selector Grid */}
        <div className="bg-cyber-panel border-2 border-cyber-border p-5 shadow-[4px_4px_0px_0px_#000] md:col-span-2">
          <div className="flex items-center gap-2 mb-4 border-b border-cyber-border/40 pb-2">
            <Type className="w-4 h-4 text-cyber-accent" />
            <h3 className="text-xs font-mono font-bold text-cyber-heading uppercase tracking-wider">
              Active Typography Engine
            </h3>
          </div>

          <div className="flex flex-col gap-4">
            <div>
              <label className="text-[10px] font-mono uppercase text-cyber-text/60 block mb-2">
                Font Family Selector (Google Fonts Loader)
              </label>
              <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2">
                {fontOptions.map((font) => (
                  <button
                    key={font}
                    onClick={() => handleFontFamilyChange(font)}
                    className={`px-3 py-2 text-xs transition-all border-2 text-left truncate cursor-pointer ${settings.fontFamily === font
                        ? 'bg-cyber-accent border-cyber-border text-black font-bold shadow-[2px_2px_0px_0px_var(--cyber-accent-glow)]'
                        : 'bg-cyber-bg border-cyber-border text-cyber-text hover:bg-white/5 hover:text-white'
                      }`}
                    style={{ fontFamily: font, borderRadius: 0 }}
                  >
                    {font}
                  </button>
                ))}
              </div>
            </div>

            <div className="border-t border-cyber-border/40 pt-4">
              <div className="flex items-center justify-between mb-2">
                <label className="text-[10px] font-mono uppercase text-cyber-text/60">
                  Global Text Scale / Font Size
                </label>
                <span className="text-xs font-mono text-cyber-accent font-bold">{settings.fontSize}px</span>
              </div>
              <input
                type="range"
                min="14"
                max="18"
                step="0.5"
                value={settings.fontSize}
                onChange={(e) => handleFontSizeChange(parseFloat(e.target.value))}
                className="w-full accent-cyber-accent h-1 bg-cyber-bg border border-cyber-border rounded-none appearance-none cursor-pointer"
              />
              <div className="flex justify-between text-[8px] font-sans text-cyber-text/40 mt-1">
                <span>14.0px</span>
                <span>16.0px</span>
                <span>18.0px</span>
              </div>
            </div>
          </div>
        </div>

      </div>

      {/* Footer Branding Info */}
      <div className="bg-cyber-panel/30 border-2 border-cyber-border/60 p-4 text-[9px] font-sans text-cyber-text/50 uppercase tracking-widest text-center mt-6">
        CLUAIZ TECHNOLOGY COGNITIVE ENGINE CONSOLE v0.1.0 — SECURE WORKSPACE LOADED
      </div>

    </div>
  );
}
