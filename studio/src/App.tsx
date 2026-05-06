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
import { Activity, Pause, Play, Layout as LayoutIcon, Shield, MessageSquare, Users, ClipboardList } from 'lucide-react';
import { AnimatePresence } from 'framer-motion';
import ThreadCard from './components/ThreadCard';
import ThreadDetail from './components/ThreadDetail';
import Lounge from './components/Lounge';
import Office from './components/Office';
import BossBoard from './components/BossBoard';
import type { Thread, Event, Agent } from './types';
import { initSocket } from './api/socket';
import { getTranslation } from './i18n';

const App: React.FC = () => {
  const [threads, setThreads] = useState<Thread[]>([]);
  const activeThreads = threads.filter(t => t.id !== 'lounge' && t.project_id !== 'system');
  const projectId = activeThreads.length > 0 ? activeThreads[0].project_id : (threads.find(t => t.id !== 'lounge')?.project_id || 'AXON-FACTORY-01');
  const [totalSignals, setTotalSignals] = useState(0);
  const [isRunning, setIsRunning] = useState(true);
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(null);
  const [events, setEvents] = useState<Event[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [locale] = useState<string>('ko_KR');
  const [activeChannel, setActiveChannel] = useState<'dashboard' | 'work' | 'office' | 'boss' | 'nogari' | 'signals'>('dashboard');
  
  const t = getTranslation(locale);

  const fetchThreads = async () => {
    try {
      const res = await fetch('http://localhost:8080/api/threads');
      const data: Thread[] = await res.json();
      
      // v0.0.23: Priority Sorting (Active Threads at Top)
      const sorted = [...data].sort((a, b) => {
        const getPriority = (status: string) => {
          if (['Working', 'SeniorReview', 'JuniorProposal', 'PatchReady'].includes(status)) return 0;
          if (['Completed'].includes(status)) return 2;
          return 1;
        };
        
        const prioA = getPriority(a.status);
        const prioB = getPriority(b.status);
        
        if (prioA !== prioB) return prioA - prioB;
        return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime();
      });
      
      setThreads(sorted);
    } catch (err) {
      console.error('Failed to fetch threads', err);
    }
  };

  const fetchAgents = async () => {
    try {
      const res = await fetch('http://localhost:8080/api/agents');
      const data = await res.json();
      setAgents(data);
    } catch (err) {
      console.error('Failed to fetch agents', err);
    }
  };

  const fetchStatus = async () => {
    try {
      const res = await fetch('http://localhost:8080/api/status');
      const data = await res.json();
      setIsRunning(data.is_running);
      setTotalSignals(data.total_signals);
    } catch (err) {
      console.error('Failed to fetch status', err);
    }
  };

  const fetchEvents = async () => {
    try {
      const res = await fetch('http://localhost:8080/api/events');
      const data = await res.json();
      // Reverse the data if backend returns DESC order (we want newest at top in UI)
      setEvents(data);
    } catch (err) {
      console.error('Failed to fetch events', err);
    }
  };

  useEffect(() => {
    fetchThreads();
    fetchAgents();
    fetchStatus();
    fetchEvents();
    
    const socket = initSocket('ws://localhost:8080');
    
    socket.onEvent((ev: any) => {
      if (ev.event_type) {
        setEvents(prev => [ev as Event, ...prev].slice(0, 50));
        
        if (ev.event_type === 'ThreadStatusChanged' || ev.event_type === 'ThreadCompleted') {
          fetchThreads();
        }
        if (ev.event_type === 'AgentAssigned' || ev.event_type === 'SystemWarning') {
          fetchAgents();
        }
      }
    });

    const interval = setInterval(() => {
        fetchThreads();
        fetchAgents();
        fetchStatus();
    }, 5000);

    return () => {
      socket.disconnect();
      clearInterval(interval);
    };
  }, []);

  const handleTogglePause = async () => {
    try {
        const endpoint = isRunning ? 'pause' : 'resume';
        await fetch(`http://localhost:8080/api/${endpoint}`, { method: 'POST' });
        setIsRunning(!isRunning);
    } catch (err) {
        console.error('Toggle pause failed', err);
    }
  };

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

  return (
    <div className="dashboard">
      <header className="header">
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <Activity color={isRunning ? 'var(--status-running)' : 'var(--status-hold)'} className={isRunning ? 'animate-pulse' : ''} />
          <h1 style={{ fontFamily: 'Orbitron', fontSize: '1.2rem', letterSpacing: '2px' }}>
            AXON <span style={{ color: 'var(--text-dim)', fontSize: '0.7rem' }}>WORKSPACE: {projectId} [{t.controlTower}]</span>
          </h1>
        </div>
        <div style={{ display: 'flex', gap: '0.5rem' }}>
          <button 
            className="btn-control" 
            onClick={handleTogglePause}
            style={{ borderColor: isRunning ? 'var(--status-hold)' : 'var(--status-running)' }}
          >
            {isRunning ? <Pause size={16} /> : <Play size={16} />} 
            {isRunning ? t.pauseFactory : t.resumeFactory}
          </button>
        </div>
      </header>

      <div className="sidebar">
        <section className="panel" style={{ padding: '0' }}>
          <div className="panel-header" style={{ opacity: 0.7, fontSize: '0.6rem' }}>{t.boardsHeader} / {t.boards}</div>
          <nav className="nav-menu">
            <button 
              className={`nav-item ${activeChannel === 'dashboard' ? 'active' : ''}`}
              onClick={() => setActiveChannel('dashboard')}
            >
              <LayoutIcon size={18} />
              <span>{t.dashboard}</span>
            </button>
            <button 
              className={`nav-item ${activeChannel === 'work' ? 'active' : ''}`}
              onClick={() => setActiveChannel('work')}
            >
              <ClipboardList size={18} />
              <span>{t.workBoard}</span>
            </button>
            <button 
              className={`nav-item ${activeChannel === 'office' ? 'active' : ''}`}
              onClick={() => setActiveChannel('office')}
            >
              <Users size={18} />
              <span>{t.office}</span>
            </button>
            <button 
              className={`nav-item ${activeChannel === 'boss' ? 'active' : ''}`}
              onClick={() => setActiveChannel('boss')}
            >
              <Shield size={18} />
              <span>{t.boss}</span>
            </button>
            <button 
              className={`nav-item ${activeChannel === 'nogari' ? 'active' : ''}`}
              onClick={() => setActiveChannel('nogari')}
            >
              <MessageSquare size={18} />
              <span>{t.nogari}</span>
            </button>
            <button 
              className={`nav-item ${activeChannel === 'signals' ? 'active' : ''}`}
              onClick={() => setActiveChannel('signals')}
            >
              <Activity size={18} />
              <span>{t.signals}</span>
            </button>
          </nav>
        </section>
      </div>

      <div className="main-content">
        {activeChannel === 'dashboard' && (
          <div style={{ display: 'grid', gridTemplateColumns: '1fr', gridTemplateRows: 'auto 1fr', gap: '1rem', height: '100%' }}>
            <section className="panel">
                <div className="panel-header">{t.factoryOverview}</div>
                <div style={{ padding: '1.5rem', display: 'flex', gap: '4rem' }}>
                    <div>
                        <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginBottom: '0.5rem' }}>{t.activeThreads}</div>
                        <div style={{ fontSize: '2.5rem', fontWeight: 'bold', color: 'var(--accent-primary)', fontFamily: 'Orbitron' }}>{threads.length}</div>
                    </div>
                    <div>
                        <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginBottom: '0.5rem' }}>{t.totalSignals}</div>
                        <div style={{ fontSize: '2.5rem', fontWeight: 'bold', color: 'var(--accent-secondary)', fontFamily: 'Orbitron' }}>{totalSignals}</div>
                    </div>
                    <div style={{ flex: 1 }}>
                        <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginBottom: '0.5rem' }}>{t.latestStatus}</div>
                        <div style={{ fontSize: '1rem', fontWeight: 'bold', borderLeft: '3px solid var(--accent-primary)', paddingLeft: '1rem' }}>
                          {events[0]?.content || t.allSystemsNominal}
                        </div>
                    </div>
                </div>
            </section>
            
            <section className="panel" style={{ overflow: 'hidden' }}>
                <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span>{t.recentStrategicThreads}</span>
                  <button className="btn-mini" onClick={() => setActiveChannel('work')}>{t.viewAll}</button>
                </div>
                <div style={{ padding: '1.5rem', display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: '1rem', overflowY: 'auto' }}>
                    {threads.filter(t => t.id !== 'lounge').slice(0, 6).map(t => (
                        <ThreadCard key={t.id} thread={t} onClick={() => setSelectedThreadId(t.id)} />
                    ))}
                    {threads.length === 0 && <div className="empty-state">{t.noThreads}</div>}
                </div>
            </section>
          </div>
        )}

        {activeChannel === 'work' && (
          <section className="panel" style={{ flex: 1 }}>
            <div className="panel-header">{t.workBoardTitle} / {projectId}</div>
            <div className="thread-grid" style={{ padding: '1.5rem', display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '1rem', overflowY: 'auto' }}>
              {/* v0.0.25: Strategic sync - use pre-filtered activeThreads for consistent display */}
              {activeThreads.map(thread => (
                <div 
                  key={thread.id} 
                  onClick={() => {
                    setSelectedThreadId(thread.id);
                    setActiveChannel('work');
                  }}
                  style={{ cursor: 'pointer' }}
                >
                  <ThreadCard thread={thread} />
                </div>
              ))}
              {activeThreads.length === 0 && <div className="empty-state">{t.noWorkThreads}</div>}
            </div>
          </section>
        )}

        {activeChannel === 'office' && <Office agents={agents} setAgents={setAgents} />}

        {activeChannel === 'boss' && <BossBoard />}
        
        {activeChannel === 'nogari' && (
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', height: '100%' }}>
            <Lounge events={events} />
          </div>
        )}

        {activeChannel === 'signals' && (
          <section className="panel" style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
            <div className="panel-header">{t.realTimeSignals}</div>
            <div style={{ flex: 1, padding: '1.5rem', overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                {events.filter(e => e.event_type !== 'MessagePosted').map(e => (
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
                {events.length === 0 && <div className="empty-state">{t.silenceInFactory}</div>}
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
