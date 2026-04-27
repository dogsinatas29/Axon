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
import { Users, Shield, User, Plus, Move, Sliders, ChevronDown, ChevronRight, X } from 'lucide-react';
import type { Agent, AgentRole } from '../types';

interface OfficeProps {
  agents: Agent[];
  setAgents: React.Dispatch<React.SetStateAction<Agent[]>>;
}

const Office: React.FC<OfficeProps> = ({ agents, setAgents }) => {
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set(['architect-agent-1', 'senior-agent-1']));

  const toggleExpand = (id: string) => {
    const newExpanded = new Set(expandedNodes);
    if (newExpanded.has(id)) newExpanded.delete(id);
    else newExpanded.add(id);
    setExpandedNodes(newExpanded);
  };

  const hireAgent = async (parentId: string | null, role: AgentRole) => {
    try {
      const response = await fetch('http://localhost:8080/api/agents/hire', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ role, parent_id: parentId }),
      });
      if (response.ok) {
        const newAgent = await response.json();
        setAgents(prev => [...prev, newAgent]);
      }
    } catch (err) {
      console.error('Hiring failed', err);
    }
  };

  const fireAgent = async (id: string) => {
    if (!window.confirm('Are you sure you want to fire this agent? If this is a Senior, all subordinates will be fired as well.')) return;
    try {
      const response = await fetch(`http://localhost:8080/api/agents/${id}/fire`, {
        method: 'POST',
      });
      if (response.ok) {
        // Remove the fired agent and ALL their descendants
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
      } else {
        const errorText = await response.text();
        alert(`Firing failed: ${errorText}`);
      }
    } catch (err) {
      console.error('Firing failed', err);
    }
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
            {agent.persona.character_core}
          </div>

          {/* Controls */}
          <div style={{ display: 'flex', gap: '0.5rem', marginTop: '0.5rem', marginLeft: '1.8rem', opacity: 0.6, fontSize: '0.7rem' }}>
            <button className="btn-mini" title="Fire Agent (Cascading)" onClick={() => fireAgent(agent.id)}>
              <X size={10} color="var(--status-hold)" />
            </button>
            <button className="btn-mini" title="Move Agent"><Move size={10} /></button>
            <button className="btn-mini" title="Add Sub-Agent" onClick={() => hireAgent(agent.id, agent.role === 'Architect' ? 'Senior' : 'Junior')}>
              <Plus size={10} />
            </button>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.3rem', marginLeft: 'auto' }}>
              <Sliders size={10} />
              <input 
                type="range" min="0" max="1" step="0.1" 
                value={agent.dtr} 
                readOnly
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

  const rootAgents = agents.filter(a => !a.parent_id).sort((a, b) => {
    const roles = ['Architect', 'Senior', 'Junior'];
    return roles.indexOf(a.role) - roles.indexOf(b.role);
  });

  return (
    <section className="panel" style={{ flex: 1, maxHeight: '600px', display: 'flex', flexDirection: 'column' }}>
      <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', alignItems: 'center' }}>
          <Users size={16} style={{ marginRight: '0.5rem' }} />
          The Office <span style={{ color: 'var(--text-dim)', marginLeft: '0.3rem' }}>/ Org Chart</span>
        </div>
        <button className="btn-mini" onClick={() => hireAgent(null, 'Architect')}><Plus size={12} /> BOSS-LEVEL</button>
      </div>
      <div style={{ padding: '1rem', overflowY: 'auto', flex: 1 }}>
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
              No hierarchy established. Hire some agents!
            </div>
          )}
        </div>
      </div>
    </section>
  );
};

export default Office;
