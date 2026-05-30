/*
 * AXON - The Automated Software Factory
 * Copyright (C) 2026 dogsinatas
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

import React, { useState } from 'react';
import { Users, Shield, User, Plus, Move, Sliders, ChevronDown, ChevronRight, X, Cloud, Server, MessageSquare } from 'lucide-react';
import type { Agent, AgentRole } from '../types';

interface OfficeProps {
  agents: Agent[];
  setAgents: React.Dispatch<React.SetStateAction<Agent[]>>;
  t: any;
}

interface PersonaConfig {
  name: string;
  age?: number;
  gender: string;
  personality: string;
  speech_style: string;
  catchphrase?: string;
}

const Office: React.FC<OfficeProps> = ({ agents, setAgents, t }) => {
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set(['architect-agent-1', 'senior-agent-1']));
  const [showPersonaModal, setShowPersonaModal] = useState<string | null>(null);
  const [personaForm, setPersonaForm] = useState<PersonaConfig>({
    name: '',
    gender: 'Male',
    personality: '',
    speech_style: '',
    catchphrase: '',
  });

  const toggleExpand = (id: string) => {
    const newExpanded = new Set(expandedNodes);
    if (newExpanded.has(id)) newExpanded.delete(id);
    else newExpanded.add(id);
    setExpandedNodes(newExpanded);
  };

  const hireAgent = async (parentId: string | null, role: AgentRole) => {
    const defaultProvider = { runtime: 'local', provider: 'ollama', model: 'qwen2.5:7b' };
    
    try {
      const response = await fetch(`/api/agents/hire`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          role,
          model: defaultProvider.model,
          runtime: defaultProvider.runtime,
          provider: defaultProvider.provider,
          persona: showPersonaModal === role ? personaForm : undefined,
        }),
      });
      if (response.ok) {
        const newAgent = await response.json();
        setAgents(prev => [...prev, { ...newAgent, role, parent_id: parentId, status: 'Idle', dtr: 0.5 }]);
        setShowPersonaModal(null);
      }
    } catch (err) {
      console.error('Hiring failed', err);
    }
  };

  const fireAgent = async (id: string, role: AgentRole) => {
    if (!window.confirm(t.fireConfirm)) return;
    try {
      const response = await fetch(`/api/agents/${id}/fire`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ role }),
      });
      if (response.ok) {
        const result = await response.json();
        setAgents(prev => {
            const toRemove = new Set([id]);
            let changed = true;
            while(changed) {
                changed = false;
                prev.forEach(a => {
                    if (a.parent_id && toRemove.has(a.parent_id) && !toRemove.has(a.id)) {
                        toRemove.add(a.id);
                        changed = true;
                    }
                });
            }
            return prev.filter(a => !toRemove.has(a.id));
        });
        if (result.eviction === 'graceful') {
          alert(`${t.evictionGraceful}: ${id}`);
        }
      } else {
        const errorText = await response.text();
        alert(`${t.firingFailed}: ${errorText}`);
      }
    } catch (err) {
      console.error('Firing failed', err);
    }
  };

  const swapProvider = async (agentId: string, newRuntime: string, newProvider: string, newModel: string) => {
    try {
      const response = await fetch(`/api/agents/${agentId}/swap-provider`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ runtime: newRuntime, provider: newProvider, model: newModel }),
      });
      if (response.ok) {
        setAgents(prev => prev.map(a => 
          a.id === agentId ? { ...a, model: newModel } : a
        ));
      }
    } catch (err) {
      console.error('Provider swap failed', err);
    }
  };

  const openPersonaModal = (agentId: string) => {
    setShowPersonaModal(agentId);
    setPersonaForm({ name: '', gender: 'Male', personality: '', speech_style: '', catchphrase: '' });
  };

  const renderProviderSelector = (agent: Agent) => (
    <div style={{ display: 'flex', gap: '0.3rem', alignItems: 'center', fontSize: '0.65rem', marginTop: '0.3rem' }}>
      {agent.model?.includes('gemini') || agent.model?.includes('claude') ? (
        <Cloud size={10} color="var(--accent-secondary)" />
      ) : (
        <Server size={10} color="var(--accent-primary)" />
      )}
      <select
        value={agent.model?.includes('gemini') ? 'gemini' : agent.model?.includes('claude') ? 'claude' : 'ollama'}
        onChange={(e) => {
          const models: Record<string, string> = {
            ollama: 'qwen2.5:7b',
            gemini: 'gemini-2.0-flash',
            claude: 'claude-3-5-sonnet',
          };
          const runtimes: Record<string, string> = {
            ollama: 'local',
            gemini: 'cloud',
            claude: 'cloud',
          };
          swapProvider(agent.id, runtimes[e.target.value], e.target.value, models[e.target.value]);
        }}
        style={{ background: 'var(--bg-secondary)', color: 'var(--text)', border: '1px solid var(--border)', borderRadius: '2px', fontSize: '0.6rem', padding: '1px 2px' }}
      >
        <option value="ollama">Ollama (Local)</option>
        <option value="gemini">Gemini (Cloud)</option>
        <option value="claude">Claude (Cloud)</option>
      </select>
    </div>
  );

  const renderPersonaBadge = (agent: Agent) => (
    <button
      className="btn-mini"
      title="Edit Nogari Persona"
      onClick={() => openPersonaModal(agent.id)}
      style={{ display: 'flex', alignItems: 'center', gap: '0.2rem' }}
    >
      <MessageSquare size={10} color="var(--accent-secondary)" />
    </button>
  );

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
                {agent.persona?.name || agent.id}
              </span>
              <span style={{ fontSize: '0.7rem', color: 'var(--accent-primary)', opacity: 0.8 }}>
                ({agent.model || 'Unknown Model'})
              </span>
              <span style={{ fontSize: '0.6rem', background: 'rgba(255,255,255,0.1)', padding: '1px 4px', borderRadius: '3px', opacity: 0.7 }}>
                {agent.role}
              </span>
            </div>
            <div className="status-dot status-running" style={{ background: agent.status === 'Idle' ? '#333' : 'var(--status-running)' }}></div>
          </div>

          <div style={{ fontSize: '0.7rem', color: 'var(--text-dim)', marginTop: '0.3rem', marginLeft: '1.8rem' }}>
            {agent.persona?.character_core || 'Standard Agent'}
          </div>

          {renderProviderSelector(agent)}

          {/* Controls */}
          <div style={{ display: 'flex', gap: '0.5rem', marginTop: '0.5rem', marginLeft: '1.8rem', opacity: 0.6, fontSize: '0.7rem' }}>
            <button className="btn-mini" title="Fire Agent (Graceful Eviction)" onClick={() => fireAgent(agent.id, agent.role as AgentRole)}>
              <X size={10} color="var(--status-hold)" />
            </button>
            <button className="btn-mini" title="Move Agent"><Move size={10} /></button>
            <button className="btn-mini" title="Add Sub-Agent" onClick={() => hireAgent(agent.id, agent.role === 'Architect' ? 'Senior' : 'Junior')}>
              <Plus size={10} />
            </button>
            {renderPersonaBadge(agent)}
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.3rem', marginLeft: 'auto' }}>
              <Sliders size={10} />
              <input 
                type="range" min="0" max="1" step="0.1" 
                value={agent.dtr || 0.5} 
                readOnly
                style={{ width: '40px', height: '4px' }}
              />
              <span style={{ fontSize: '0.6rem' }}>{t.dtrLabel}: {agent.dtr?.toFixed(1) || '0.5'}</span>
            </div>
          </div>
        </div>
        {isExpanded && children.map(child => renderAgentNode(child, level + 1))}
      </div>
    );
  };

  const rootAgents = agents.filter(a => !a.parent_id).sort((a, b) => {
    const roles = ['Architect', 'Senior', 'Junior'];
    return roles.indexOf(a.role) - roles.indexOf(b.role);
  });

  return (
    <section className="panel" style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
      <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', alignItems: 'center' }}>
          <Users size={16} style={{ marginRight: '0.5rem' }} />
          {t.office} <span style={{ color: 'var(--text-dim)', marginLeft: '0.3rem' }}>/ {t.orgChart}</span>
        </div>
        <button className="btn-mini" onClick={() => hireAgent(null, 'Architect')}><Plus size={12} /> {t.hireBossLevel}</button>
      </div>
      <div style={{ padding: '1rem', overflowY: 'auto', flex: 1 }}>
        <div style={{ borderLeft: '2px solid rgba(255,255,255,0.05)', paddingLeft: '0.5rem' }}>
          <div style={{ marginBottom: '1rem', textAlign: 'center', opacity: 0.8 }}>
             <div style={{ display: 'inline-block', padding: '0.5rem 1rem', border: '1px solid var(--accent-secondary)', borderRadius: '4px', fontSize: '0.9rem', fontWeight: 'bold', background: 'rgba(112, 0, 255, 0.1)' }}>
               {t.bossYou}
             </div>
             <div style={{ height: '20px', width: '2px', background: 'var(--accent-secondary)', margin: '0 auto' }}></div>
          </div>
          {rootAgents.map(agent => renderAgentNode(agent))}
          {agents.length === 0 && (
            <div style={{ textAlign: 'center', color: 'var(--text-dim)', fontSize: '0.8rem', marginTop: '2rem' }}>
              {t.noHierarchy}
            </div>
          )}
        </div>
      </div>

      {showPersonaModal && (
        <div style={{
          position: 'fixed', top: 0, left: 0, right: 0, bottom: 0,
          background: 'rgba(0,0,0,0.7)', display: 'flex', alignItems: 'center', justifyContent: 'center',
          zIndex: 1000
        }}>
          <div className="card" style={{ width: '400px', padding: '1.5rem' }}>
            <h3 style={{ marginBottom: '1rem' }}>Nogari.md Persona Injection</h3>
            <p style={{ fontSize: '0.7rem', color: 'var(--text-dim)', marginBottom: '1rem' }}>
              This persona is ONLY used in Lounge/Nogari channels. Code generation remains pure.
            </p>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
              <label style={{ fontSize: '0.7rem' }}>
                Name:
                <input
                  type="text"
                  value={personaForm.name}
                  onChange={(e) => setPersonaForm(prev => ({ ...prev, name: e.target.value }))}
                  style={{ width: '100%', background: 'var(--bg-secondary)', border: '1px solid var(--border)', color: 'var(--text)', padding: '0.3rem' }}
                />
              </label>
              <label style={{ fontSize: '0.7rem' }}>
                Age:
                <input
                  type="number"
                  value={personaForm.age || ''}
                  onChange={(e) => setPersonaForm(prev => ({ ...prev, age: parseInt(e.target.value) || undefined }))}
                  style={{ width: '100%', background: 'var(--bg-secondary)', border: '1px solid var(--border)', color: 'var(--text)', padding: '0.3rem' }}
                />
              </label>
              <label style={{ fontSize: '0.7rem' }}>
                Gender:
                <select
                  value={personaForm.gender}
                  onChange={(e) => setPersonaForm(prev => ({ ...prev, gender: e.target.value }))}
                  style={{ width: '100%', background: 'var(--bg-secondary)', border: '1px solid var(--border)', color: 'var(--text)', padding: '0.3rem' }}
                >
                  <option value="Male">Male</option>
                  <option value="Female">Female</option>
                  <option value="Non-binary">Non-binary</option>
                </select>
              </label>
              <label style={{ fontSize: '0.7rem' }}>
                Personality:
                <input
                  type="text"
                  value={personaForm.personality}
                  onChange={(e) => setPersonaForm(prev => ({ ...prev, personality: e.target.value }))}
                  placeholder="e.g., Cynical, Coffee-addict"
                  style={{ width: '100%', background: 'var(--bg-secondary)', border: '1px solid var(--border)', color: 'var(--text)', padding: '0.3rem' }}
                />
              </label>
              <label style={{ fontSize: '0.7rem' }}>
                Speech Style:
                <input
                  type="text"
                  value={personaForm.speech_style}
                  onChange={(e) => setPersonaForm(prev => ({ ...prev, speech_style: e.target.value }))}
                  placeholder="e.g., Casual, Formal, Dialect"
                  style={{ width: '100%', background: 'var(--bg-secondary)', border: '1px solid var(--border)', color: 'var(--text)', padding: '0.3rem' }}
                />
              </label>
              <label style={{ fontSize: '0.7rem' }}>
                Catchphrase:
                <input
                  type="text"
                  value={personaForm.catchphrase || ''}
                  onChange={(e) => setPersonaForm(prev => ({ ...prev, catchphrase: e.target.value }))}
                  placeholder="e.g., 'That's not how we did it in my day...'"
                  style={{ width: '100%', background: 'var(--bg-secondary)', border: '1px solid var(--border)', color: 'var(--text)', padding: '0.3rem' }}
                />
              </label>
            </div>
            <div style={{ display: 'flex', gap: '0.5rem', marginTop: '1rem', justifyContent: 'flex-end' }}>
              <button className="btn-mini" onClick={() => setShowPersonaModal(null)}>Cancel</button>
              <button className="btn-mini" style={{ background: 'var(--accent-secondary)' }} onClick={() => setShowPersonaModal(null)}>
                Apply Persona (Nogari Only)
              </button>
            </div>
          </div>
        </div>
      )}
    </section>
  );
};

export default Office;
