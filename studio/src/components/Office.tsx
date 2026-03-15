import React, { useState } from 'react';
import { Users, Shield, User, Plus, Edit2, Move, Sliders, ChevronDown, ChevronRight } from 'lucide-react';
import type { Agent, AgentRole } from '../types';

interface OfficeProps {
  agents: Agent[];
  setAgents: React.Dispatch<React.SetStateAction<Agent[]>>;
}

const Office: React.FC<OfficeProps> = ({ agents, setAgents }) => {
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set(['architect-1', 'senior-1']));

  const toggleExpand = (id: string) => {
    const newExpanded = new Set(expandedNodes);
    if (newExpanded.has(id)) newExpanded.delete(id);
    else newExpanded.add(id);
    setExpandedNodes(newExpanded);
  };

  const updateDTR = (id: string, value: number) => {
    setAgents(prev => prev.map(a => a.id === id ? { ...a, dtr: value } : a));
  };

  const addAgent = (parentId: string | null, role: AgentRole) => {
    const newAgent: Agent = {
      id: `agent-${Date.now()}`,
      role,
      parent_id: parentId || undefined,
      status: 'Idle',
      dtr: 0.5,
      persona: {
        name: 'New Agent',
        character_core: 'Fresh Talent',
        prefixes: [],
        suffixes: [],
        description: 'Just arrived at the factory.'
      }
    };
    setAgents(prev => [...prev, newAgent]);
  };

  const renderAgentNode = (agent: Agent, level: number = 0) => {
    const children = agents.filter(a => a.parent_id === agent.id);
    const isExpanded = expandedNodes.has(agent.id);
    const hasChildren = children.length > 0;

    return (
      <div key={agent.id} style={{ marginLeft: `${level * 1.5}rem`, marginBottom: '0.5rem' }}>
        <div className="card" style={{ 
          borderLeft: `4px solid ${agent.role === 'Architect' ? '#fff' : agent.role === 'Senior' ? 'var(--accent-secondary)' : 'var(--accent-primary)'}`,
          padding: '0.75rem',
          position: 'relative'
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              {hasChildren ? (
                <button onClick={() => toggleExpand(agent.id)} style={{ background: 'none', border: 'none', color: 'var(--text-dim)', cursor: 'pointer', padding: 0 }}>
                  {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                </button>
              ) : <div style={{ width: 14 }} />}
              
              {agent.role === 'Architect' ? <Shield size={14} color="#fff" /> : 
               agent.role === 'Senior' ? <Shield size={14} color="var(--accent-secondary)" /> : 
               <User size={14} color="var(--accent-primary)" />}
              
              <span style={{ fontWeight: 'bold', fontSize: '0.85rem' }}>
                {agent.persona.name}
              </span>
              <span style={{ fontSize: '0.6rem', background: 'rgba(255,255,255,0.1)', padding: '1px 4px', borderRadius: '3px', opacity: 0.7 }}>
                {agent.role}
              </span>
            </div>
            <div className="status-dot status-running" style={{ background: agent.status === 'Idle' ? '#333' : 'var(--status-running)' }}></div>
          </div>

          <div style={{ fontSize: '0.7rem', color: 'var(--text-dim)', marginTop: '0.3rem', marginLeft: '1.8rem' }}>
            {agent.persona.character_core}
          </div>

          {/* Controls */}
          <div style={{ display: 'flex', gap: '0.5rem', marginTop: '0.5rem', marginLeft: '1.8rem', opacity: 0.6, fontSize: '0.7rem' }}>
            <button className="btn-mini" title="Edit Persona"><Edit2 size={10} /></button>
            <button className="btn-mini" title="Move Agent"><Move size={10} /></button>
            <button className="btn-mini" title="Add Sub-Agent" onClick={() => addAgent(agent.id, agent.role === 'Architect' ? 'Senior' : 'Junior')}>
              <Plus size={10} />
            </button>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.3rem', marginLeft: 'auto' }}>
              <Sliders size={10} />
              <input 
                type="range" min="0" max="1" step="0.1" 
                value={agent.dtr} 
                onChange={(e) => updateDTR(agent.id, parseFloat(e.target.value))} 
                style={{ width: '40px', height: '4px' }}
              />
              <span style={{ fontSize: '0.6rem' }}>DTR: {agent.dtr}</span>
            </div>
          </div>
        </div>
        {isExpanded && children.map(child => renderAgentNode(child, level + 1))}
      </div>
    );
  };

  const rootAgents = agents.filter(a => !a.parent_id);

  return (
    <section className="panel" style={{ flex: 1, maxHeight: '600px' }}>
      <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', alignItems: 'center' }}>
          <Users size={16} style={{ marginRight: '0.5rem' }} />
          The Office <span style={{ color: 'var(--text-dim)', marginLeft: '0.3rem' }}>/ Org Chart</span>
        </div>
        <button className="btn-mini" onClick={() => addAgent(null, 'Architect')}><Plus size={12} /> BOSS-LEVEL</button>
      </div>
      <div style={{ padding: '1rem', overflowY: 'auto' }}>
        <div style={{ borderLeft: '2px solid rgba(255,255,255,0.05)', paddingLeft: '0.5rem' }}>
          <div style={{ marginBottom: '1rem', textAlign: 'center', opacity: 0.8 }}>
             <div style={{ display: 'inline-block', padding: '0.5rem 1rem', border: '1px solid var(--accent-secondary)', borderRadius: '4px', fontSize: '0.9rem', fontWeight: 'bold', background: 'rgba(112, 0, 255, 0.1)' }}>
               BOSS (YOU)
             </div>
             <div style={{ height: '20px', width: '2px', background: 'var(--accent-secondary)', margin: '0 auto' }}></div>
          </div>
          {rootAgents.map(agent => renderAgentNode(agent))}
          {agents.length === 0 && (
            <div style={{ textAlign: 'center', color: 'var(--text-dim)', fontSize: '0.8rem', marginTop: '2rem' }}>
              No hierarchy established.
            </div>
          )}
        </div>
      </div>
    </section>
  );
};

export default Office;
