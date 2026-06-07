import React, { useState } from 'react';
import { useDbStore } from '../../../store/useDbStore';
import { Card } from '../../../components/ui/Card';
import { Button } from '../../../components/ui/Button';
import { Database, Play, Trash2 } from 'lucide-react';

export const DbManager: React.FC = () => {
  const activeEngine = useDbStore((state) => state.activeDbEngine);
  const setActiveEngine = useDbStore((state) => state.setActiveDbEngine);
  const logs = useDbStore((state) => state.logs);
  const addLog = useDbStore((state) => state.addLog);
  const clearLogs = useDbStore((state) => state.clearLogs);
  const setSandboxStatus = useDbStore((state) => state.setSandboxStatus);

  const [payloadInput, setPayloadInput] = useState('');
  const [selectedType, setSelectedType] = useState('text');

  const handleCreateNeuron = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!payloadInput.trim()) return;

    setSandboxStatus('simulating');
    addLog('info', `Simulating write execution in ephemeral sandbox`);

    // Simulate validation gate check
    setTimeout(() => {
      // 90% pass rate
      const pass = Math.random() > 0.1;
      if (pass) {
        setSandboxStatus('passed');
        addLog('success', `Deep Archer verified new neuron. Committed to physical storage.`);
      } else {
        setSandboxStatus('failed');
        addLog('error', `Failed validation! Neuron write aborted (NaN vector check failed).`);
      }
      setPayloadInput('');
    }, 1500);
  };

  const handleEngineChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const val = e.target.value as any;
    setActiveEngine(val);
    addLog('info', `Switched Database mode: ${val} adapter engine active`);
  };

  return (
    <Card className="flex flex-col h-full hover:border-cyber-neonBlue/40 transition-all duration-300">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-cyber-border pb-2 mb-4">
        <div className="flex items-center gap-2">
          <Database className="w-5 h-5 text-cyber-neonBlue" />
          <span className="font-mono text-xs uppercase tracking-wider text-cyber-text font-bold">
            Database Engine Controller
          </span>
        </div>
        <select
          value={activeEngine}
          onChange={handleEngineChange}
          className="bg-cyber-bg border border-cyber-border rounded px-2 py-0.5 text-xs text-cyber-neonBlue font-mono outline-none"
        >
          <option value="LMDB">LMDB Core</option>
          <option value="SQLite">SQLite Sync</option>
          <option value="Postgres">Postgres Pool</option>
          <option value="MongoDB">MongoDB Atlas</option>
          <option value="PDF_Doc_Parser">PDF Ingester</option>
        </select>
      </div>

      {/* Write Ingestion Form */}
      <form onSubmit={handleCreateNeuron} className="flex flex-col gap-3 mb-4">
        <span className="text-[10px] font-mono uppercase text-cyber-text/60 tracking-wider">
          Ingest New Neuron
        </span>
        <div className="flex gap-2">
          <input
            type="text"
            placeholder="Type payload string (e.g. text/hex)..."
            value={payloadInput}
            onChange={(e) => setPayloadInput(e.target.value)}
            className="flex-1 bg-cyber-bg border border-cyber-border rounded-lg px-3 py-2 text-xs font-mono text-white outline-none focus:border-cyber-neonBlue"
          />
          <select
            value={selectedType}
            onChange={(e) => setSelectedType(e.target.value)}
            className="bg-cyber-bg border border-cyber-border rounded-lg px-2 text-xs font-mono text-cyber-text"
          >
            <option value="text">Text</option>
            <option value="voltage">Voltage</option>
            <option value="audio">Audio</option>
            <option value="pdf">PDF</option>
          </select>
          <Button type="submit" variant="primary" className="px-3">
            <Play className="w-3.5 h-3.5" />
          </Button>
        </div>
      </form>

      {/* Logs section */}
      <div className="flex-1 flex flex-col min-h-[140px]">
        <div className="flex items-center justify-between border-b border-cyber-border/40 pb-1 mb-2">
          <span className="text-[10px] font-mono uppercase text-cyber-text/60 tracking-wider">
            System Console Output
          </span>
          <button
            onClick={clearLogs}
            className="text-cyber-text/30 hover:text-cyber-neonPink transition-colors flex items-center gap-1 text-[9px] font-mono uppercase"
          >
            <Trash2 className="w-3 h-3" /> Clear
          </button>
        </div>

        <div className="flex-1 bg-cyber-bg/40 border border-cyber-border/40 rounded-lg p-2 overflow-y-auto max-h-[160px] flex flex-col gap-1.5 font-mono text-xs text-cyber-text">
          {logs.map((log, i) => (
            <div key={i} className="flex gap-2 items-start leading-tight">
              <span className="text-cyber-text/35 text-[10px] select-none">{log.time}</span>
              <span className={`uppercase font-extrabold text-[10px] select-none ${
                log.type === 'error' ? 'text-cyber-neonPink' :
                log.type === 'warn' ? 'text-cyber-neonYellow' :
                log.type === 'success' ? 'text-cyber-neonGreen' : 'text-cyber-neonBlue'
              }`}>
                [{log.type}]
              </span>
              <span className="flex-1 text-cyber-text/90 break-all">{log.message}</span>
            </div>
          ))}
        </div>
      </div>
    </Card>
  );
};
