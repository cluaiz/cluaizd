import { LayoutDashboard, Settings, Database, Shield, Brain } from 'lucide-react';
import { useDbStore } from '../../store/useDbStore';

export function PrimaryRail() {
  const activeView = useDbStore((state) => state.activeView);
  const setActiveView = useDbStore((state) => state.setActiveView);

  const navItems = [
    { id: 'dashboard', label: 'Command Center', icon: LayoutDashboard },
    { id: 'graph', label: 'Synapse Graph', icon: Brain },
    { id: 'database', label: 'Database Console', icon: Database },
    { id: 'sandbox', label: 'Archer Gate', icon: Shield },
    { id: 'settings', label: 'Tuning Settings', icon: Settings },
  ] as const;

  return (
    <aside className="w-[72px] h-full flex flex-col items-center bg-cyber-panel/95 border-r-2 border-cyber-border select-none flex-shrink-0">
      
      {/* Top Header Logo Section */}
      <div className="h-16 w-full flex items-center justify-center border-b-2 border-cyber-border/80 select-none">
        <span className="text-2xl leading-none">🪼</span>
      </div>

      {/* Navigation Icons */}
      <div className="flex-1 flex flex-col gap-5 w-full px-2 mt-6">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = activeView === item.id;

          return (
            <button
              key={item.id}
              onClick={() => setActiveView(item.id)}
              title={item.label}
              className={`w-full aspect-square flex items-center justify-center border-2 transition-all relative group cursor-pointer ${
                isActive
                  ? 'bg-cyber-accent/15 border-cyber-accent text-cyber-accent hover:shadow-[3px_3px_0px_0px_var(--cyber-accent-glow)] active:shadow-[1px_1px_0px_0px_var(--cyber-accent-glow)]'
                  : 'bg-transparent border-transparent text-cyber-text/60 hover:text-cyber-heading hover:bg-white/5 hover:border-cyber-border hover:shadow-[3px_3px_0px_0px_var(--cyber-accent-glow)] active:shadow-[1px_1px_0px_0px_var(--cyber-accent-glow)]'
              }`}
              style={{ borderRadius: 0 }}
            >
              <Icon className="w-5 h-5" />

              {/* Active Indicator Bar */}
              {isActive && (
                <div className="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-6 bg-cyber-accent" />
              )}
            </button>
          );
        })}
      </div>

      {/* Bottom User Indicator */}
      <div className="w-10 h-10 mb-6 bg-cyber-bg border-2 border-cyber-border flex items-center justify-center text-xs font-mono text-cyber-text/80 shadow-[2px_2px_0px_0px_#000]">
        U
      </div>
    </aside>
  );
}
