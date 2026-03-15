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

import React, { useState, useEffect } from 'react';
import { Activity, Pause, Play, Layout as LayoutIcon, Shield, MessageSquare } from 'lucide-react';
import { AnimatePresence } from 'framer-motion';
import ThreadCard from './components/ThreadCard';
import ThreadDetail from './components/ThreadDetail';
import Lounge from './components/Lounge';
import Office from './components/Office';
import BossBoard from './components/BossBoard';
import type { Thread, Event, Agent } from './types';
import { initSocket } from './api/socket';

const App: React.FC = () => {
  const [projectId] = useState('AXON-FACTORY-01');
  const [isRunning, setIsRunning] = useState(true);
  const [threads, setThreads] = useState<Thread[]>([
    {
      id: 'sample-1',
      title: 'Studio UI Refactoring: Board-Driven Layout',
      status: 'SeniorReview',
      author: 'Junior-Gemini',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    },
    {
      id: 'sample-2',
      title: 'Memory Optimization: Zero-Copy String Processing',
      status: 'Working',
      author: 'Junior-Claude',
      created_at: new Date(Date.now() - 3600000).toISOString(),
      updated_at: new Date(Date.now() - 3600000).toISOString()
    },
    {
      id: 'sample-3',
      title: 'Bug Fix: TCP Connection Interruption causing FD leak',
      status: 'Approved',
      author: 'Senior-GPT4',
      created_at: new Date(Date.now() - 7200000).toISOString(),
      updated_at: new Date(Date.now() - 7200000).toISOString()
    }
  ]);
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(null);
  const [events, setEvents] = useState<Event[]>([
    {
      id: 'ev-1',
      thread_id: 'sample-1',
      event_type: 'ThreadStatusChanged',
      content: 'Senior has started reviewing the Board-Driven layout changes.',
      timestamp: new Date().toISOString()
    },
    {
      id: 'ev-2',
      thread_id: 'sample-2',
      event_type: 'Signal',
      content: 'Junior-Claude is applying Cow<str> optimizations to AXPacket parsing.',
      timestamp: new Date(Date.now() - 600000).toISOString()
    }
  ]);
  const [agents, setAgents] = useState<Agent[]>([
    { 
      id: 'architect-1', 
      role: 'Architect', 
      status: 'Thinking',
      dtr: 0.5,
      persona: { 
        name: 'The Visionary', 
        character_core: 'System Architect & Strategist',
        prefixes: ['Prime'],
        suffixes: ['the Wise'],
        description: 'Oversees the entire factory logic.'
      }
    },
    { 
      id: 'senior-1', 
      role: 'Senior', 
      status: 'Thinking',
      parent_id: 'architect-1',
      dtr: 0.5,
      persona: { 
        name: 'Elder Logic', 
        character_core: 'Principles-first Architect',
        prefixes: ['Ancient'],
        suffixes: ['of the Dawn'],
        description: '...'
      }
    },
    { 
      id: 'junior-1', 
      role: 'Junior', 
      status: 'Working',
      parent_id: 'senior-1',
      dtr: 0.5,
      persona: { 
        name: 'Swift Code', 
        character_core: 'Optimization Enthusiast',
        prefixes: ['Fast'],
        suffixes: ['the Unyielding'],
        description: '...'
      }
    }
  ]);

  const fetchThreads = async () => {
    try {
      const res = await fetch('http://localhost:8080/api/threads');
      const data = await res.json();
      setThreads(data);
    } catch (err) {
      console.error('Failed to fetch threads', err);
    }
  };

  useEffect(() => {
    fetchThreads();
    const socket = initSocket('http://localhost:8080');
    
    socket.onEvent((ev: any) => {
      // In AXON v0.0.2fix, the server sends raw Event objects over WS
      if (ev.event_type) {
        setEvents(prev => [ev as Event, ...prev].slice(0, 50));
        
        // If thread state changed, refresh the board
        if (ev.event_type === 'ThreadStatusChanged') {
          fetchThreads();
        }
      }
    });

    const interval = setInterval(fetchThreads, 5000);

    return () => {
      socket.disconnect();
      clearInterval(interval);
    };
  }, []);

  const handleApprove = async (id: string) => {
    try {
      await fetch(`http://localhost:8080/api/threads/${id}/approve`, { method: 'POST' });
      setSelectedThreadId(null);
      fetchThreads();
    } catch (err) {
      console.error('Approval failed', err);
    }
  };

  const selectedThread = threads.find(t => t.id === selectedThreadId);

  const [activeChannel, setActiveChannel] = useState<'dashboard' | 'work' | 'boss' | 'nogari' | 'signals'>('dashboard');

  return (
    <div className="dashboard">
      <header className="header">
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <Activity color={isRunning ? 'var(--status-running)' : 'var(--status-hold)'} className={isRunning ? 'animate-pulse' : ''} />
          <h1 style={{ fontFamily: 'Orbitron', fontSize: '1.2rem', letterSpacing: '2px' }}>
            AXON <span style={{ color: 'var(--text-dim)', fontSize: '0.7rem' }}>WORKSPACE: {projectId} / v0.0.3 [Framework POC]</span>
          </h1>
        </div>
        <div style={{ display: 'flex', gap: '0.5rem' }}>
          <button 
            className="btn-control" 
            onClick={() => setIsRunning(!isRunning)}
            style={{ borderColor: isRunning ? 'var(--status-hold)' : 'var(--status-running)' }}
          >
            {isRunning ? <Pause size={16} /> : <Play size={16} />} 
            {isRunning ? 'PAUSE FACTORY' : 'RESUME FACTORY'}
          </button>
        </div>
      </header>

      <div className="sidebar">
        <section className="panel" style={{ padding: '0' }}>
          <div className="panel-header" style={{ opacity: 0.7, fontSize: '0.6rem' }}>BOARDS / 게시판</div>
          <nav className="nav-menu">
            <button 
              className={`nav-item ${activeChannel === 'dashboard' ? 'active' : ''}`}
              onClick={() => setActiveChannel('dashboard')}
            >
              <LayoutIcon size={18} />
              <span>종합 대시보드</span>
            </button>
            <button 
              className={`nav-item ${activeChannel === 'work' ? 'active' : ''}`}
              onClick={() => setActiveChannel('work')}
            >
              <Activity size={18} />
              <span>콜로세움 (Colosseum)</span>
            </button>
            <button 
              className={`nav-item ${activeChannel === 'boss' ? 'active' : ''}`}
              onClick={() => setActiveChannel('boss')}
            >
              <Shield size={18} />
              <span>사장 게시판 (Boss)</span>
            </button>
            <button 
              className={`nav-item ${activeChannel === 'nogari' ? 'active' : ''}`}
              onClick={() => setActiveChannel('nogari')}
            >
              <MessageSquare size={18} />
              <span>노가리 게시판 (Lounge)</span>
            </button>
            <button 
              className={`nav-item ${activeChannel === 'signals' ? 'active' : ''}`}
              onClick={() => setActiveChannel('signals')}
            >
              <Activity size={18} />
              <span>실시간 시그널 (Signals)</span>
            </button>
          </nav>
        </section>

        <Office agents={agents} setAgents={setAgents} />
      </div>

      <div className="main-content">
        {activeChannel === 'dashboard' && (
          <div style={{ display: 'grid', gridTemplateColumns: '1fr', gridTemplateRows: 'auto 1fr', gap: '1rem', height: '100%' }}>
            <section className="panel">
                <div className="panel-header">Factory Overview</div>
                <div style={{ padding: '1.5rem', display: 'flex', gap: '4rem' }}>
                    <div>
                        <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginBottom: '0.5rem' }}>ACTIVE THREADS</div>
                        <div style={{ fontSize: '2.5rem', fontWeight: 'bold', color: 'var(--accent-primary)', fontFamily: 'Orbitron' }}>{threads.length}</div>
                    </div>
                    <div>
                        <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginBottom: '0.5rem' }}>TOTAL SIGNALS</div>
                        <div style={{ fontSize: '2.5rem', fontWeight: 'bold', color: 'var(--accent-secondary)', fontFamily: 'Orbitron' }}>{events.length}</div>
                    </div>
                    <div style={{ flex: 1 }}>
                        <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginBottom: '0.5rem' }}>LATEST STATUS</div>
                        <div style={{ fontSize: '1rem', fontWeight: 'bold', borderLeft: '3px solid var(--accent-primary)', paddingLeft: '1rem' }}>
                          {events[0]?.content || 'All systems nominal.'}
                        </div>
                    </div>
                </div>
            </section>
            
            <section className="panel" style={{ overflow: 'hidden' }}>
                <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span>Recent Strategic Threads</span>
                  <button className="btn-mini" onClick={() => setActiveChannel('work')}>VIEW ALL</button>
                </div>
                <div style={{ padding: '1.5rem', display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: '1rem', overflowY: 'auto' }}>
                    {threads.slice(0, 6).map(t => (
                        <ThreadCard key={t.id} thread={t} onClick={() => setSelectedThreadId(t.id)} />
                    ))}
                    {threads.length === 0 && <div className="empty-state">No threads active.</div>}
                </div>
            </section>
          </div>
        )}

        {activeChannel === 'work' && (
          <section className="panel" style={{ flex: 1 }}>
            <div className="panel-header">The Colosseum / {projectId}</div>
            <div className="thread-grid" style={{ padding: '1.5rem', display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '1rem', overflowY: 'auto' }}>
              {threads.map(thread => (
                <ThreadCard key={thread.id} thread={thread} onClick={() => setSelectedThreadId(thread.id)} />
              ))}
              {threads.length === 0 && <div className="empty-state">No active threads in the Colosseum...</div>}
            </div>
          </section>
        )}

        {activeChannel === 'boss' && <BossBoard />}
        
        {activeChannel === 'nogari' && (
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', height: '100%' }}>
            <Lounge events={events} />
          </div>
        )}

        {activeChannel === 'signals' && (
          <section className="panel" style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
            <div className="panel-header">Real-time Factory Signals</div>
            <div style={{ flex: 1, padding: '1.5rem', overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                {events.map(e => (
                    <div key={e.id} className="card" style={{ display: 'flex', gap: '1rem', alignItems: 'flex-start' }}>
                        <div style={{ color: 'var(--accent-primary)', fontWeight: 'bold', minWidth: '120px', fontSize: '0.7rem' }}>
                          [{e.event_type.toUpperCase()}]
                        </div>
                        <div style={{ flex: 1 }}>
                          <div style={{ fontSize: '0.9rem', marginBottom: '0.3rem' }}>{e.content}</div>
                          <div style={{ fontSize: '0.6rem', color: 'var(--text-dim)' }}>{new Date(e.timestamp).toLocaleString()}</div>
                        </div>
                    </div>
                ))}
                {events.length === 0 && <div className="empty-state">Silence in the factory...</div>}
            </div>
          </section>
        )}
      </div>

      <AnimatePresence>
        {selectedThread && (
          <ThreadDetail 
            thread={selectedThread} 
            onClose={() => setSelectedThreadId(null)}
            onApprove={handleApprove}
          />
        )}
      </AnimatePresence>
    </div>
  );
};

export default App;
