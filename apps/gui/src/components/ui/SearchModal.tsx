import React, { useState, useEffect, useRef } from 'react';
import { Search, Terminal, Sliders, Globe, Trash2, ShieldAlert } from 'lucide-react';
import { useDbStore } from '../../store/useDbStore';

export function SearchModal() {
  const isSearchModalOpen = useDbStore((state) => state.isSearchModalOpen);
  const setSearchModalOpen = useDbStore((state) => state.setSearchModalOpen);
  const setActiveView = useDbStore((state) => state.setActiveView);
  const clearLogs = useDbStore((state) => state.clearLogs);
  const setSandboxStatus = useDbStore((state) => state.setSandboxStatus);
  const addLog = useDbStore((state) => state.addLog);
  const nodes = useDbStore((state) => state.nodes);
  const setSelectedNode = useDbStore((state) => state.setSelectedNode);

  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus input on open
  useEffect(() => {
    if (isSearchModalOpen) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isSearchModalOpen]);

  // Window listeners for Ctrl + K and Escape
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K')) {
        e.preventDefault();
        setSearchModalOpen(!isSearchModalOpen);
      } else if (e.key === 'Escape' && isSearchModalOpen) {
        e.preventDefault();
        setSearchModalOpen(false);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isSearchModalOpen, setSearchModalOpen]);

  if (!isSearchModalOpen) return null;

  // Filter commands and database nodes
  const allCommands = [
    { 
      type: 'command', 
      id: 'goto-dashboard', 
      label: '/dashboard', 
      desc: 'Navigate to System Overview Matrix', 
      icon: Terminal, 
      action: () => { setActiveView('dashboard'); addLog('info', 'Command: Navigated to Dashboard'); }
    },
    { 
      type: 'command', 
      id: 'goto-settings', 
      label: '/settings', 
      desc: 'Open Kernel Spacing and Theme Settings', 
      icon: Sliders, 
      action: () => { setActiveView('settings'); addLog('info', 'Command: Navigated to Settings'); }
    },
    { 
      type: 'command', 
      id: 'clear-console', 
      label: '/clear', 
      desc: 'Flush DB console logs registry', 
      icon: Trash2, 
      action: () => { clearLogs(); addLog('success', 'Console logs flushed successfully'); }
    },
    { 
      type: 'command', 
      id: 'sim-sandbox', 
      label: '/sim', 
      desc: 'Simulate Deep Archer Security verification gate', 
      icon: ShieldAlert, 
      action: () => { setSandboxStatus('simulating'); addLog('info', 'Command: Triggered security gate simulation'); }
    },
  ];

  const matchedCommands = allCommands.filter(c => 
    c.label.toLowerCase().includes(query.toLowerCase()) || 
    c.desc.toLowerCase().includes(query.toLowerCase())
  );

  const matchedNodes = query.trim() === '' ? [] : nodes.filter(n => 
    n.name?.toLowerCase().includes(query.toLowerCase()) ||
    n.label?.toLowerCase().includes(query.toLowerCase())
  ).map(n => ({
    type: 'node',
    id: n.id,
    label: n.name,
    desc: `Node Type: ${n.label} | Cluster: ${n.cluster || 'N/A'}`,
    icon: Globe,
    action: () => { setSelectedNode(n); setActiveView('dashboard'); addLog('info', `Selected node: ${n.name}`); }
  }));

  const results = [...matchedCommands, ...matchedNodes];

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((prev) => (prev + 1) % results.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((prev) => (prev - 1 + results.length) % results.length);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (results[selectedIndex]) {
        results[selectedIndex].action();
        setSearchModalOpen(false);
      }
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-24 px-4 bg-black/75 backdrop-blur-sm select-none">
      {/* Click Outside to Close */}
      <div className="fixed inset-0 -z-10" onClick={() => setSearchModalOpen(false)} />

      {/* Main Command Box */}
      <div 
        onKeyDown={handleKeyDown}
        className="w-full max-w-lg bg-cyber-panel border-2 border-cyber-border shadow-[8px_8px_0px_0px_#000] flex flex-col"
        style={{ borderRadius: 0 }}
      >
        {/* Search Input Field */}
        <div className="flex items-center gap-3 px-4 py-3 border-b-2 border-cyber-border bg-cyber-bg/40">
          <Search className="w-4 h-4 text-cyber-neonBlue shrink-0" />
          <input
            ref={inputRef}
            type="text"
            placeholder="Type a command (/) or search database neurons..."
            value={query}
            onChange={(e) => { setQuery(e.target.value); setSelectedIndex(0); }}
            className="flex-1 bg-transparent text-xs font-mono text-white outline-none placeholder-cyber-text/30"
          />
          <span className="text-[9px] font-mono bg-cyber-border px-1.5 py-0.5 border border-cyber-border text-cyber-text/50">
            ESC
          </span>
        </div>

        {/* Results List */}
        <div className="max-h-[320px] overflow-y-auto p-2 flex flex-col gap-1 [&::-webkit-scrollbar]:hidden">
          {results.length > 0 ? (
            results.map((item, idx) => {
              const Icon = item.icon;
              const isSelected = selectedIndex === idx;
              return (
                <button
                  key={item.id}
                  onClick={() => { item.action(); setSearchModalOpen(false); }}
                  className={`w-full flex items-center gap-3 px-3 py-2 text-left transition-all border ${
                    isSelected
                      ? 'bg-cyber-neonBlue/15 border-cyber-neonBlue text-cyber-neonBlue shadow-[2px_2px_0px_0px_rgba(0,240,255,0.4)]'
                      : 'bg-transparent border-transparent text-cyber-text/80 hover:bg-white/5'
                  }`}
                  style={{ borderRadius: 0 }}
                >
                  <Icon className={`w-4 h-4 shrink-0 ${isSelected ? 'text-cyber-neonBlue' : 'text-cyber-text/40'}`} />
                  <div className="flex-1 min-w-0">
                    <div className="text-xs font-mono font-bold truncate">{item.label}</div>
                    <div className={`text-[9px] font-mono uppercase truncate ${isSelected ? 'text-cyber-neonBlue/80' : 'text-cyber-text/50'}`}>
                      {item.desc}
                    </div>
                  </div>
                  {item.type === 'command' && (
                    <span className="text-[9px] font-mono bg-cyber-border/40 px-1 py-0.5 border border-cyber-border text-cyber-text/60 shrink-0">
                      CMD
                    </span>
                  )}
                </button>
              );
            })
          ) : (
            <div className="py-8 text-center text-xs font-mono text-cyber-text/30 uppercase tracking-wider">
              No matching neurons or commands found
            </div>
          )}
        </div>

        {/* Info Footer */}
        <div className="px-4 py-2 border-t border-cyber-border/40 bg-cyber-bg/25 flex items-center justify-between text-[9px] font-mono text-cyber-text/40 uppercase tracking-widest">
          <span>↑↓ navigation | enter select</span>
          <span>ctrl+k toggle</span>
        </div>

      </div>
    </div>
  );
}
