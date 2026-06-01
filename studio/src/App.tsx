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
  const [nogariCount, setNogariCount] = useState(0);
  const [activeWorkers, setActiveWorkers] = useState(0);
  const [bootstrapStage, setBootstrapStage] = useState('Idle');
  const [isRunning, setIsRunning] = useState(true);
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(null);
  const [events, setEvents] = useState<Event[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [locale, setLocale] = useState<string>('en_US');
  const [activeChannel, setActiveChannel] = useState<'dashboard' | 'work' | 'office' | 'boss' | 'nogari' | 'signals'>('dashboard');
  const [showSuccessPopup, setShowSuccessPopup] = useState(false);
  const [completedPhase, setCompletedPhase] = useState<number | null>(null);
  const [shownPopups, setShownPopups] = useState<{ [key: number]: boolean }>({});
  const [activeStreams, setActiveStreams] = useState<Record<string, string>>({});

  const [projectFolder, setProjectFolder] = useState<string>('');

  // 프로젝트별 팝업 상태 분리 (localStorage 키에 projectFolder 포함)
  useEffect(() => {
    fetch('/api/status')
      .then(res => res.json())
      .then(data => {
        if (data.project_folder) {
          setProjectFolder(data.project_folder);
          const key = `axon_shown_popups_${data.project_folder}`;
          const saved = localStorage.getItem(key);
          setShownPopups(saved ? JSON.parse(saved) : { 1: false, 2: false, 3: false });
        }
      });
  }, []);

  const markPopupShown = (phase: number) => {
    const next = { ...shownPopups, [phase]: true };
    setShownPopups(next);
    localStorage.setItem(`axon_shown_popups_${projectFolder}`, JSON.stringify(next));
  };

  const [bossPendingCount, setBossPendingCount] = useState(0);
  const [workInProgressCount, setWorkInProgressCount] = useState(0);
  const [currentPhase, setCurrentPhase] = useState(0);
  
  const t = getTranslation(locale);

  // v0.0.31 Session 18-Final: 모든 핵심 아키텍처 단위 구현 태스크가 Completed 되었는지 실시간 완결 정합 판정
  const getPhaseTasks = (phase: number) => {
    return activeThreads.filter(th => {
      const kind = th.task_kind;
      let kindStr = '';
      if (typeof kind === 'string') {
        kindStr = kind;
      } else if (typeof kind === 'object' && kind !== null) {
        const values = Object.values(kind);
        if (values.length > 0) {
          kindStr = values[0] as string;
        }
      }
      if (phase === 1) {
        return kindStr === 'HeaderDecl' || kindStr === 'ModuleDecl';
      } else if (phase === 2) {
        return kindStr === 'SourceImpl' || kindStr === 'ModuleImpl';
      } else {
        return kindStr === 'Integrator' || kindStr === 'IntegratorGen';
      }
    });
  };

  const getPhaseTasksFromList = (threads: Thread[], phase: number) => {
    return threads.filter(th => {
      const kind = th.task_kind;
      let kindStr = '';
      if (typeof kind === 'string') {
        kindStr = kind;
      } else if (typeof kind === 'object' && kind !== null) {
        const values = Object.values(kind);
        if (values.length > 0) {
          kindStr = values[0] as string;
        }
      }
      if (phase === 1) {
        return kindStr === 'HeaderDecl' || kindStr === 'ModuleDecl';
      } else if (phase === 2) {
        return kindStr === 'SourceImpl' || kindStr === 'ModuleImpl';
      } else {
        return kindStr === 'Integrator' || kindStr === 'IntegratorGen';
      }
    });
  };

  const isPhase1Complete = getPhaseTasks(1).length > 0 && getPhaseTasks(1).every(th => th.status === 'Completed');
  const isPhase2Complete = getPhaseTasks(2).length > 0 && getPhaseTasks(2).every(th => th.status === 'Completed');
  const isPhase3Complete = getPhaseTasks(3).length > 0 && getPhaseTasks(3).every(th => th.status === 'Completed');

  // v0.0.31.38: [FIX_PHASE_POPUP] Track popup shown state independently from localStorage.
  // localStorage is only for persistence across page reloads. The runtime state ensures
  // the popup shows every time a phase completes, even if localStorage says it was shown before.
  const [popupShownRuntime, setPopupShownRuntime] = useState<{ [key: number]: boolean }>({ 1: false, 2: false, 3: false });

  useEffect(() => {
    // v0.0.31.38: [FIX_PHASE_POPUP] Wait for projectFolder to be loaded before checking popups.
    // Without this guard, shownPopups is {} on first render and the popup check races with
    // the localStorage load, causing the popup to never appear.
    if (!projectFolder) return;

    // Runtime state takes priority over localStorage — always show popup on phase completion
    if (isPhase3Complete && !popupShownRuntime[3]) {
      setCompletedPhase(3);
      setShowSuccessPopup(true);
      setPopupShownRuntime(prev => ({ ...prev, 3: true }));
      markPopupShown(3);
    } else if (isPhase2Complete && !popupShownRuntime[2]) {
      setCompletedPhase(2);
      setShowSuccessPopup(true);
      setPopupShownRuntime(prev => ({ ...prev, 2: true }));
      markPopupShown(2);
    } else if (isPhase1Complete && !popupShownRuntime[1]) {
      setCompletedPhase(1);
      setShowSuccessPopup(true);
      setPopupShownRuntime(prev => ({ ...prev, 1: true }));
      markPopupShown(1);
    }
  }, [isPhase1Complete, isPhase2Complete, isPhase3Complete, projectFolder]);

  // v0.0.31.38: [FIX_PHASE_POPUP] Reset runtime popup flags when phase becomes incomplete
  // (e.g., user retried tasks or restarted the pipeline).
  useEffect(() => {
    setPopupShownRuntime(prev => {
      let next = { ...prev };
      let changed = false;
      if (!isPhase1Complete && prev[1]) { next[1] = false; changed = true; }
      if (!isPhase2Complete && prev[2]) { next[2] = false; changed = true; }
      if (!isPhase3Complete && prev[3]) { next[3] = false; changed = true; }
      return changed ? next : prev;
    });
  }, [isPhase1Complete, isPhase2Complete, isPhase3Complete]);

  const getSuccessPopupContent = () => {
    if (completedPhase === 1) {
      return {
        title: locale === 'ko_KR' ? '🏆 Phase 1 (설계 선언) 공정 완료!' : locale === 'ja_JP' ? '🏆 Phase 1 (設計宣言) 工程完了！' : '🏆 Phase 1 (Spec & Design) Complete!',
        desc: locale === 'ko_KR' ? '모든 아키텍처 명세 및 헤더 프로토타입 선언(HeaderGen) 공정이 성공적으로 승인 및 완료되었습니다.' : locale === 'ja_JP' ? 'すべてのアーキテクチャ仕様およびヘッダープロトタイプ宣言（HeaderGen）工程が正常に承認および完了しました。' : 'All architecture specifications and header declarations (HeaderGen) have been successfully authorized and completed.'
      };
    } else if (completedPhase === 2) {
      return {
        title: locale === 'ko_KR' ? '🏆 Phase 2 (소스 구현) 공정 완료!' : locale === 'ja_JP' ? '🏆 Phase 2 (ソース実装) 工程完了！' : '🏆 Phase 2 (Source Implementation) Complete!',
        desc: locale === 'ko_KR' ? '모든 핵심 비즈니스 로직 및 소스 코드 파일(ImplGen) 구현이 컴파일 오류 없이 안전하게 빌드 완료되었습니다.' : locale === 'ja_JP' ? 'すべての主要ビジネスロジックおよびソースコードファイル（ImplGen）の実装がコンパイルエラーなしで正常にビルドされました。' : 'All core business logic and source code implementations (ImplGen) have been successfully built with zero compilation errors.'
      };
    } else {
      return {
        title: t.factorySuccessTitle || "🏆 Factory Manufacturing Cycle Complete!",
        desc: t.factorySuccessDesc || "All strategic modules compiled and verified successfully."
      };
    }
  };

  const popupContent = getSuccessPopupContent();

  const fetchThreads = async () => {
    try {
      const res = await fetch(`/api/threads`);
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

        // Within same status, sort by Phase (task_kind)
        const getPhaseOrder = (kind: any) => {
            if (!kind) return 3;
            let kindStr = '';
            if (typeof kind === 'string') {
                kindStr = kind;
            } else if (typeof kind === 'object') {
                const values = Object.values(kind);
                if (values.length > 0) {
                    kindStr = values[0] as string;
                }
            }
            if (kindStr === 'HeaderDecl' || kindStr === 'ModuleDecl') return 1;
            if (kindStr === 'SourceImpl' || kindStr === 'ModuleImpl') return 2;
            return 3;
        };

        const phaseA = getPhaseOrder(a.task_kind);
        const phaseB = getPhaseOrder(b.task_kind);

        if (phaseA !== phaseB) return phaseA - phaseB;

        return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime();
      });
      
      setThreads(sorted);
    } catch (err) {
      console.error('Failed to fetch threads', err);
    }
  };

  const fetchAgents = async () => {
    try {
      const res = await fetch(`/api/agents`);
      const data = await res.json();
      setAgents(data);
    } catch (err) {
      console.error('Failed to fetch agents', err);
    }
  };

  const fetchStatus = async () => {
    try {
      const res = await fetch(`/api/status`);
      const data = await res.json();
      setIsRunning(data.is_running);
      setTotalSignals(data.total_signals);
      setNogariCount(data.nogari_count);
      setActiveWorkers(data.active_threads);
      setBootstrapStage(data.bootstrap_stage);
      if (data.locale) {
        setLocale(data.locale);
      }
    } catch (err) {
      console.error('Failed to fetch status', err);
    }
  };

  const fetchEvents = async () => {
    try {
      const res = await fetch(`/api/events`);
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
    
    // v0.0.30: Fixed redundant /ws suffix — use Vite proxy
    const wsProto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const socket = initSocket(`${wsProto}//${window.location.hostname}:${window.location.port}`);
    
    socket.onEvent((ev: any) => {
      if (ev.event_type) {
        // v0.0.31.38: [FIX_SIGNAL_FLOOD] AgentStreamingData goes ONLY to activeStreams (ThreadDetail)
        // NOT to the events array — prevents thousands of signal-card renders in the Signals tab.
        if (ev.event_type === 'AgentStreamingData') {
          setActiveStreams(prev => {
            const threadId = ev.thread_id || 'unknown';
            const chunk = ev.payload?.chunk || '';
            const existing = prev[threadId] || '';
            return { ...prev, [threadId]: existing + chunk };
          });
          return;  // Skip events array — streaming data is for ThreadDetail only
        }
        
        setEvents(prev => [ev as Event, ...prev].slice(0, 50));
        
        if (ev.event_type === 'AgentResponse' || ev.event_type === 'PostAdded') {
          if (ev.thread_id) {
            setActiveStreams(prev => {
              const next = { ...prev };
              delete next[ev.thread_id];
              return next;
            });
          }
        }
        
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

  useEffect(() => {
    const checkBossPending = async () => {
      let count = 0;
      try {
        const specRes = await fetch(`/api/specs/approval`);
        if (specRes.ok) {
          const specData = await specRes.json();
          if (!specData.approved && !specData.rejected) {
            count += 1;
          }
        }
      } catch {}
      
      try {
        const reviewRes = await fetch(`/api/pipeline/reviews`);
        if (reviewRes.ok) {
          const reviews = await reviewRes.json();
          count += reviews.length;
        }
      } catch {}
      
      try {
        const threadsRes = await fetch(`/api/threads`);
        if (threadsRes.ok) {
          const threadsData: Thread[] = await threadsRes.json();
          // Exclude BossApproval threads from Boss Board badge count as they are processed in Work Board detail modal
          // const bossApprovalCount = threadsData.filter(t => t.status === 'BossApproval').length;
          // count += bossApprovalCount;
          
          const inProgressCount = threadsData.filter(t => t.status === 'Working').length;
          setWorkInProgressCount(inProgressCount);
          
          const phase1Done = getPhaseTasksFromList(threadsData, 1).length > 0 && getPhaseTasksFromList(threadsData, 1).every(t => t.status === 'Completed');
          const phase2Done = getPhaseTasksFromList(threadsData, 2).length > 0 && getPhaseTasksFromList(threadsData, 2).every(t => t.status === 'Completed');
          const phase3Done = getPhaseTasksFromList(threadsData, 3).length > 0 && getPhaseTasksFromList(threadsData, 3).every(t => t.status === 'Completed');
          if (phase3Done) setCurrentPhase(3);
          else if (phase2Done) setCurrentPhase(2);
          else if (phase1Done) setCurrentPhase(1);
          else if (getPhaseTasksFromList(threadsData, 2).length > 0) setCurrentPhase(2);
          else if (getPhaseTasksFromList(threadsData, 1).length > 0) setCurrentPhase(1);
        }
      } catch {}
      
      setBossPendingCount(count);
    };
    
    checkBossPending();
    const interval = setInterval(checkBossPending, 2000);
    return () => clearInterval(interval);
  }, []);

  const handleTogglePause = async () => {
    try {
        const endpoint = isRunning ? 'pause' : 'resume';
        await fetch(`/api/${endpoint}`, { method: 'POST' });
        setIsRunning(!isRunning);
    } catch (err) {
        console.error('Toggle pause failed', err);
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
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <div style={{
            display: 'flex', alignItems: 'center', gap: '0.4rem',
            fontSize: '0.7rem', color: 'var(--text-dim)',
            padding: '0.3rem 0.6rem', borderRadius: '6px',
            background: 'rgba(255,255,255,0.03)',
            border: '1px solid rgba(255,255,255,0.06)'
          }}>
            <span style={{ fontWeight: 'bold', color: activeWorkers > 0 ? 'var(--accent-primary)' : 'var(--text-dim)' }}>
              👷 {activeWorkers}
            </span>
            <span>{t.workers || 'Workers'}</span>
          </div>
          <div style={{
            display: 'flex', alignItems: 'center', gap: '0.4rem',
            fontSize: '0.7rem', color: 'var(--text-dim)',
            padding: '0.3rem 0.6rem', borderRadius: '6px',
            background: 'rgba(255,255,255,0.03)',
            border: '1px solid rgba(255,255,255,0.06)'
          }}>
            <span className="badge" style={{
              background: bootstrapStage === 'Complete' ? 'rgba(16, 185, 129, 0.2)' :
                           bootstrapStage.includes('Running') ? 'rgba(59, 130, 246, 0.2)' :
                           'rgba(255,255,255,0.05)',
              color: bootstrapStage === 'Complete' ? '#10b981' :
                     bootstrapStage.includes('Running') ? '#3b82f6' :
                     'var(--text-dim)',
              fontSize: '0.6rem',
              padding: '2px 6px',
              borderRadius: '4px',
              fontFamily: 'monospace'
            }}>{bootstrapStage}</span>
          </div>
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
              style={{ position: 'relative' }}
            >
              <LayoutIcon size={18} />
              <span>{t.dashboard}</span>
              {currentPhase > 0 && (
                <span style={{
                  position: 'absolute',
                  top: '4px',
                  right: '8px',
                  background: 'rgba(255,255,255,0.1)',
                  color: '#aaa',
                  fontSize: '0.55rem',
                  fontWeight: 'bold',
                  padding: '1px 4px',
                  borderRadius: '6px'
                }}>
                  P{currentPhase}/3
                </span>
              )}
            </button>
            <button 
              className={`nav-item ${activeChannel === 'work' ? 'active' : ''}`}
              onClick={() => setActiveChannel('work')}
              style={{ position: 'relative' }}
            >
              <ClipboardList size={18} />
              <span>{t.workBoard}</span>
              {workInProgressCount > 0 && (
                <span style={{
                  position: 'absolute',
                  top: '4px',
                  right: '8px',
                  background: '#3b82f6',
                  color: '#fff',
                  fontSize: '0.6rem',
                  fontWeight: 'bold',
                  padding: '1px 5px',
                  borderRadius: '10px',
                  minWidth: '16px',
                  textAlign: 'center',
                  animation: 'workPulse 2s ease-in-out infinite',
                  boxShadow: '0 0 8px rgba(59, 130, 246, 0.6)'
                }}>
                  {workInProgressCount}
                </span>
              )}
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
              style={{ position: 'relative' }}
            >
              <Shield size={18} />
              <span>{t.boss}</span>
              {bossPendingCount > 0 && (
                <span style={{
                  position: 'absolute',
                  top: '4px',
                  right: '8px',
                  background: '#ff4444',
                  color: '#fff',
                  fontSize: '0.6rem',
                  fontWeight: 'bold',
                  padding: '1px 5px',
                  borderRadius: '10px',
                  minWidth: '16px',
                  textAlign: 'center',
                  animation: 'bossPulse 1.5s ease-in-out infinite',
                  boxShadow: '0 0 8px rgba(255, 68, 68, 0.6)'
                }}>
                  {bossPendingCount}
                </span>
              )}
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
          <div style={{ display: 'grid', gridTemplateColumns: '1fr', gridTemplateRows: isPhase3Complete ? 'auto auto 1fr' : 'auto 1fr', gap: '1rem', height: '100%' }}>
            {isPhase3Complete && (
              <div className="card" style={{
                background: 'linear-gradient(135deg, rgba(16, 185, 129, 0.1) 0%, rgba(5, 150, 105, 0.2) 100%)',
                border: '1px solid rgba(16, 185, 129, 0.4)',
                boxShadow: '0 8px 32px 0 rgba(16, 185, 129, 0.2), inset 0 0 12px rgba(16, 185, 129, 0.1)',
                borderRadius: '12px',
                padding: '1.5rem',
                display: 'flex',
                alignItems: 'center',
                gap: '1.5rem',
                position: 'relative',
                overflow: 'hidden'
              }}>
                <div style={{
                  background: 'rgba(16, 185, 129, 0.2)',
                  borderRadius: '50%',
                  padding: '0.8rem',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  boxShadow: '0 0 20px rgba(16, 185, 129, 0.4)'
                }}>
                  <span style={{ fontSize: '2rem' }}>🎉</span>
                </div>
                <div style={{ flex: 1 }}>
                  <h2 style={{
                    fontFamily: 'Orbitron',
                    fontSize: '1.2rem',
                    fontWeight: 'bold',
                    color: '#10b981',
                    textShadow: '0 0 10px rgba(16, 185, 129, 0.5)',
                    marginBottom: '0.4rem',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '0.5rem'
                  }}>
                    {t.factorySuccessTitle || "🏆 Factory Cycle Complete!"}
                    <span className="badge" style={{
                      background: 'rgba(16, 185, 129, 0.2)',
                      color: '#10b981',
                      border: '1px solid #10b981',
                      fontSize: '0.6rem',
                      fontFamily: 'monospace',
                      padding: '2px 6px',
                      borderRadius: '4px'
                    }}>100% PASS</span>
                  </h2>
                  <p style={{
                    fontSize: '0.85rem',
                    color: 'rgba(255,255,255,0.7)',
                    lineHeight: '1.4'
                  }}>
                    {t.factorySuccessDesc || "All strategic modules compiled and verified successfully."}
                  </p>
                </div>
                <div style={{
                  position: 'absolute',
                  right: '15px',
                  bottom: '-10px',
                  fontSize: '4rem',
                  fontWeight: 900,
                  color: 'rgba(16, 185, 129, 0.08)',
                  pointerEvents: 'none',
                  fontFamily: 'Orbitron',
                  letterSpacing: '4px'
                }}>
                  SUCCESS
                </div>
              </div>
            )}

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
                    <div>
                        <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginBottom: '0.5rem' }}>WORKERS</div>
                        <div style={{ fontSize: '2.5rem', fontWeight: 'bold', color: 'var(--accent-primary)', fontFamily: 'Orbitron' }}>{activeWorkers}</div>
                    </div>
                    <div>
                        <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginBottom: '0.5rem' }}>NOGARI</div>
                        <div style={{ fontSize: '2.5rem', fontWeight: 'bold', color: 'var(--accent-tertiary)', fontFamily: 'Orbitron' }}>{nogariCount}</div>
                    </div>
                    <div style={{ flex: 1 }}>
                        <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginBottom: '0.5rem' }}>{t.latestStatus}</div>
                        <div style={{ fontSize: '1rem', fontWeight: 'bold', borderLeft: '3px solid var(--accent-primary)', paddingLeft: '1rem' }}>
                          {events[0]?.content || t.allSystemsNominal}
                        </div>
                    </div>
                </div>
            </section>
            
            <section className="panel" style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}>
                <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span>{t.recentStrategicThreads}</span>
                  <button className="btn-mini" onClick={() => setActiveChannel('work')}>{t.viewAll}</button>
                </div>
                <div style={{ padding: '1rem', display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: '0.6rem', overflowY: 'visible' }}>
                    {threads.filter(t => t.id !== 'lounge').slice(0, 6).map(th => (
                        <ThreadCard key={th.id} thread={th} onClick={() => setSelectedThreadId(th.id)} t={t} />
                    ))}
                    {threads.length === 0 && <div className="empty-state">{t.noThreads}</div>}
                </div>
            </section>
          </div>
        )}

        {activeChannel === 'work' && (
          <section className="panel" style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}>
            <div className="panel-header">{t.workBoardTitle} / {projectId}</div>
            <div className="thread-grid" style={{ padding: '0.8rem', display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gridAutoRows: 'max-content', alignContent: 'start', gap: '0.5rem', overflowY: 'auto', flex: 1, minHeight: 0 }}>
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
                  <ThreadCard 
                    thread={thread} 
                    t={t} 
                    onClick={(id) => {
                      setSelectedThreadId(id);
                      setActiveChannel('work');
                    }} 
                  />
                </div>
              ))}
              {activeThreads.length === 0 && <div className="empty-state">{t.noWorkThreads}</div>}
            </div>
          </section>
        )}

        {activeChannel === 'office' && <Office agents={agents} setAgents={setAgents} t={t} />}

        {activeChannel === 'boss' && <BossBoard threads={threads} events={events} t={t} />}
        
        {activeChannel === 'nogari' && (
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', height: '100%' }}>
            <Lounge events={events} t={t} />
          </div>
        )}

        {activeChannel === 'signals' && (
          <section className="panel" style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
            <div className="panel-header">{t.realTimeSignals}</div>
            <div style={{ flex: 1, padding: '1.5rem', overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                {events.filter(e => e.event_type !== 'MessagePosted').map(e => (
                    <div 
                      key={e.id} 
                      className={`card signal-card ${e.level?.toLowerCase() || 'info'}`} 
                      style={{ display: 'flex', gap: '1rem', alignItems: 'flex-start' }}
                    >
                        <div style={{ 
                          color: e.level === 'Critical' ? 'var(--status-error)' : 'var(--accent-primary)', 
                          fontWeight: 'bold', 
                          minWidth: '120px', 
                          fontSize: '0.7rem' 
                        }}>
                          [{e.event_type.toUpperCase()}] {e.level === 'Critical' && '🚨'}
                        </div>
                        <div style={{ flex: 1 }}>
                          <div style={{ 
                            fontSize: '0.9rem', 
                            marginBottom: '0.3rem',
                            fontWeight: e.level === 'Critical' || e.level === 'Error' ? 'bold' : 'normal'
                          }}>
                            {e.content}
                          </div>
                          <div style={{ fontSize: '0.6rem', color: 'var(--text-dim)' }}>
                            {e.source} | {new Date(e.timestamp).toLocaleString()}
                          </div>
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
            activeStream={activeStreams[selectedThread.id]}
            onClose={() => setSelectedThreadId(null)} 
            onRefresh={fetchThreads}
            t={t}
          />
        )}
      </AnimatePresence>

      {/* 🏆 공정 완결 기념 장엄한 성공 팝업 모달 */}
      {showSuccessPopup && (
        <div style={{
          position: 'fixed',
          top: 0,
          left: 0,
          width: '100vw',
          height: '100vh',
          backgroundColor: 'rgba(0, 0, 0, 0.8)',
          backdropFilter: 'blur(12px)',
          zIndex: 9999,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          animation: 'fadeIn 0.3s ease-out'
        }}>
          <div style={{
            width: '90%',
            maxWidth: '550px',
            background: 'linear-gradient(135deg, rgba(20, 20, 20, 0.98) 0%, rgba(10, 10, 10, 0.99) 100%)',
            border: '2px solid rgba(16, 185, 129, 0.7)',
            boxShadow: '0 0 50px rgba(16, 185, 129, 0.45), inset 0 0 20px rgba(16, 185, 129, 0.25)',
            borderRadius: '16px',
            padding: '2.5rem',
            textAlign: 'center',
            position: 'relative',
            overflow: 'hidden'
          }}>
            {/* radial gradient background glow */}
            <div style={{
              position: 'absolute',
              top: '-50%',
              left: '-50%',
              width: '200%',
              height: '200%',
              background: 'radial-gradient(circle, rgba(16, 185, 129, 0.08) 0%, transparent 60%)',
              pointerEvents: 'none',
              zIndex: 1
            }} />
            
            <div style={{ position: 'relative', zIndex: 2 }}>
              <div style={{
                fontSize: '4.5rem',
                marginBottom: '1rem',
                display: 'inline-block',
                filter: 'drop-shadow(0 0 15px rgba(16, 185, 129, 0.6))'
              }}>
                🏆
              </div>
              
              <h2 style={{
                fontFamily: 'Orbitron',
                fontSize: '1.6rem',
                fontWeight: 'bold',
                color: '#10b981',
                textShadow: '0 0 15px rgba(16, 185, 129, 0.6)',
                marginBottom: '1rem',
                letterSpacing: '1px'
              }}>
                {popupContent.title}
              </h2>
              
              <div style={{
                width: '80px',
                height: '2px',
                background: 'linear-gradient(90deg, transparent, #10b981, transparent)',
                margin: '0.8rem auto 1.5rem auto'
              }} />
              
              <p style={{
                fontSize: '0.95rem',
                color: 'rgba(255, 255, 255, 0.85)',
                lineHeight: '1.6',
                marginBottom: '2rem',
                wordBreak: 'keep-all'
              }}>
                {popupContent.desc}
              </p>

              <button 
                onClick={() => setShowSuccessPopup(false)}
                style={{
                  background: 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
                  color: '#ffffff',
                  border: 'none',
                  borderRadius: '8px',
                  padding: '0.8rem 2.5rem',
                  fontSize: '1rem',
                  fontWeight: 'bold',
                  cursor: 'pointer',
                  boxShadow: '0 0 20px rgba(16, 185, 129, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.2)',
                  transition: 'all 0.2s ease',
                  fontFamily: 'Orbitron, sans-serif',
                  letterSpacing: '1px'
                }}
              >
                {locale === 'ko_KR' ? '확인 완료 🫡' : locale === 'ja_JP' ? '確認完了 🫡' : 'CONFIRMED 🫡'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default App;
