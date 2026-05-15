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
import { Terminal, Send, ShieldCheck, AlertCircle, Cpu } from 'lucide-react';
import { type Thread, type Event } from '../types';

interface BossBoardProps {
  threads: Thread[];
  events: Event[];
  t: any;
}

const BossBoard: React.FC<BossBoardProps> = ({ threads, events, t }) => {
  const [spec, setSpec] = useState('');
  const [selectedInterruptId, setSelectedInterruptId] = useState<string | null>(null);
  const [clarification, setClarification] = useState('');
  const [semanticClosure, setSemanticClosure] = useState<any>(null);
  const [selectedRisk, setSelectedRisk] = useState<any>(null);

  // Fetch semantic risks periodically
  React.useEffect(() => {
    const fetchRisks = async () => {
      try {
        const response = await fetch('http://localhost:8080/api/semantics/risks');
        if (response.ok) {
          const data = await response.ok ? await response.json() : null;
          setSemanticClosure(data);
        }
      } catch (err) {
        console.error('Failed to fetch semantic risks:', err);
      }
    };
    fetchRisks();
    const interval = setInterval(fetchRisks, 3000);
    return () => clearInterval(interval);
  }, []);

  // v0.0.29: Filter for "ALERT" tasks (3+ rejections or specifically marked as interrupts)
  const interrupts = threads.filter(th => (th.rejection_count || 0) >= 3);

  const handleSemanticDecision = async (action: string) => {
    if (!selectedRisk) return;
    try {
      const response = await fetch('http://localhost:8080/api/semantics/decide', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          risk_id: selectedRisk.target,
          action,
          comment: clarification || 'Sealed by Boss'
        }),
      });
      if (response.ok) {
        setSelectedRisk(null);
        setClarification('');
      }
    } catch (err) {
      console.error('Failed to submit semantic decision:', err);
    }
  };

  const handleSubmitSpec = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!spec.trim()) return;

    try {
      const response = await fetch('http://localhost:8080/api/specs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: spec }),
      });

      if (response.ok) {
        alert(t.specSubmitted || 'Spec submitted successfully.');
        setSpec('');
      }
    } catch (error) {
      console.error('Error submitting spec:', error);
    }
  };

  const handleIssueClarification = async () => {
    if (!selectedInterruptId || !clarification.trim()) return;

    try {
        const response = await fetch(`http://localhost:8080/api/threads/${selectedInterruptId}/clarify`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ command: clarification }),
        });

        if (response.ok) {
            alert(t.clarificationIssued || 'Clarification issued to factory.');
            setClarification('');
            setSelectedInterruptId(null);
        }
    } catch (error) {
        console.error('Error issuing clarification:', error);
    }
  };

  return (
    <section className="panel" style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
      <div className="panel-header" style={{ background: 'var(--status-error)', color: 'white' }}>
        <ShieldCheck size={16} style={{ marginRight: '0.5rem' }} />
        {t.bossBoardTitle} - Semantic Arbitration Console (v0.0.30)
      </div>
      
      <div style={{ padding: '1.5rem', flex: 1, display: 'flex', flexDirection: 'column', gap: '1.5rem', overflowY: 'auto' }}>
        
        {/* 1. Semantic Interrupts (NEW in v0.0.30) */}
        <div className="card" style={{ border: semanticClosure?.risks?.length > 0 ? '2px solid var(--status-error)' : '1px solid rgba(255,255,255,0.1)', background: 'rgba(255,0,0,0.02)' }}>
            <h3 style={{ fontSize: '1rem', marginBottom: '1.2rem', color: 'var(--status-error)', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                <AlertCircle size={18} />
                SEMANTIC INTERRUPTS {semanticClosure?.risks?.length > 0 && `(${semanticClosure.risks.length})`}
            </h3>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: '1.5rem', minHeight: '300px' }}>
                {/* Left: Risk List */}
                <div style={{ borderRight: '1px solid rgba(255,255,255,0.05)', paddingRight: '1rem' }}>
                    {semanticClosure?.risks?.map((risk: any, i: number) => {
                        const isResolved = semanticClosure.decisions.some((d: any) => d.risk_id === risk.target);
                        const isSelected = selectedRisk?.target === risk.target;
                        return (
                            <div 
                                key={i}
                                onClick={() => setSelectedRisk(risk)}
                                style={{
                                    padding: '0.8rem',
                                    marginBottom: '0.5rem',
                                    background: isSelected ? 'rgba(255,0,0,0.2)' : 'rgba(255,255,255,0.03)',
                                    border: `1px solid ${isSelected ? 'var(--status-error)' : 'rgba(255,255,255,0.1)'}`,
                                    borderRadius: '4px',
                                    cursor: 'pointer',
                                    opacity: isResolved ? 0.5 : 1
                                }}
                            >
                                <div style={{ fontSize: '0.8rem', fontWeight: 'bold', display: 'flex', justifyContent: 'space-between' }}>
                                    <span>{risk.kind}</span>
                                    {isResolved && <ShieldCheck size={12} color="var(--status-success)" />}
                                </div>
                                <div style={{ fontSize: '0.7rem', opacity: 0.7 }}>{risk.target}</div>
                            </div>
                        );
                    })}
                    {(!semanticClosure?.risks || semanticClosure.risks.length === 0) && (
                        <div style={{ textAlign: 'center', color: 'var(--text-dim)', fontSize: '0.8rem', paddingTop: '2rem' }}>
                            No unresolved semantic risks.
                        </div>
                    )}
                </div>

                {/* Right: Detail & Arbitration */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                    {selectedRisk ? (
                        <>
                            <div style={{ padding: '1rem', background: '#000', border: '1px solid #333', borderRadius: '4px', flex: 1, overflowY: 'auto' }}>
                                <div style={{ fontSize: '0.9rem', color: 'var(--status-error)', fontWeight: 'bold', marginBottom: '0.5rem' }}>
                                    {selectedRisk.message}
                                </div>
                                <div style={{ fontSize: '0.8rem', color: '#888', marginBottom: '1rem' }}>
                                    Target: {selectedRisk.target}
                                </div>
                                <pre style={{ fontSize: '0.75rem', background: '#111', padding: '0.8rem', borderRadius: '4px', color: '#00ff88', borderLeft: '2px solid #00ff88' }}>
                                    {selectedRisk.context}
                                </pre>
                            </div>

                            <textarea 
                                value={clarification}
                                onChange={(e) => setClarification(e.target.value)}
                                placeholder="Decision context (optional)..."
                                style={{ height: '60px', background: 'rgba(0,0,0,0.3)', border: '1px solid #333', color: 'white', padding: '0.5rem', fontSize: '0.8rem' }}
                            />

                            <div style={{ display: 'flex', gap: '0.5rem' }}>
                                <button className="btn-control" onClick={() => handleSemanticDecision('SEAL')} style={{ flex: 1, background: '#00e5ff', color: '#000', fontWeight: 'bold' }}>SEAL STRUCT</button>
                                <button className="btn-control" onClick={() => handleSemanticDecision('EXCLUDE')} style={{ flex: 1, background: '#ff4444', color: 'white' }}>EXCLUDE</button>
                                <button className="btn-control" onClick={() => handleSemanticDecision('APPROVE')} style={{ flex: 1, background: '#00ff88', color: '#000' }}>APPROVE OPTIONAL</button>
                                <button className="btn-control" onClick={() => handleSemanticDecision('STOP')} style={{ flex: 1, background: '#333', color: 'white' }}>STOP</button>
                            </div>
                        </>
                    ) : (
                        <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--text-dim)', fontSize: '0.85rem', background: 'rgba(0,0,0,0.1)', borderRadius: '4px' }}>
                            Select a risk to begin arbitration.
                        </div>
                    )}
                </div>
            </div>
        </div>

        {/* 2. Legacy Failure Interrupts (3+ rejections) */}
        {interrupts.length > 0 && (
            <div className="card" style={{ border: '1px solid var(--status-error)', background: 'rgba(255,0,0,0.01)' }}>
                <h3 style={{ fontSize: '0.9rem', marginBottom: '1rem', color: 'var(--status-error)' }}>
                    REWORK INTERRUPTS ({interrupts.length})
                </h3>
                {/* ... existing logic for interrupts ... */}
                <div style={{ display: 'flex', gap: '0.5rem', overflowX: 'auto' }}>
                    {interrupts.map(th => (
                        <div key={th.id} onClick={() => setSelectedInterruptId(th.id)} style={{ padding: '0.5rem 1rem', background: 'rgba(255,0,0,0.1)', border: '1px solid rgba(255,0,0,0.2)', borderRadius: '4px', cursor: 'pointer', fontSize: '0.75rem' }}>
                            {th.title}
                        </div>
                    ))}
                </div>
            </div>
        )}
            
            {/* Horizontal list of problem tasks */}
            <div style={{ display: 'flex', gap: '0.8rem', overflowX: 'auto', paddingBottom: '1rem', marginBottom: interrupts.length > 0 ? '1rem' : 0 }}>
                {interrupts.map(th => {
                    const isActive = selectedInterruptId === th.id;
                    return (
                        <div 
                            key={th.id}
                            onClick={() => setSelectedInterruptId(isActive ? null : th.id)}
                            style={{
                                minWidth: '220px',
                                padding: '1rem',
                                background: isActive ? 'rgba(255,0,0,0.15)' : 'rgba(255,0,0,0.05)',
                                border: `1px solid ${isActive ? 'var(--status-error)' : 'rgba(255,0,0,0.15)'}`,
                                borderRadius: '6px',
                                cursor: 'pointer',
                                transition: 'all 0.2s cubic-bezier(0.4, 0, 0.2, 1)',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '0.8rem',
                                boxShadow: isActive ? '0 4px 12px rgba(255,0,0,0.15)' : 'none'
                            }}
                        >
                            <AlertCircle size={18} color="var(--status-error)" className={isActive ? 'animate-pulse' : ''} />
                            <div style={{ flex: 1, overflow: 'hidden' }}>
                                <div style={{ fontSize: '0.85rem', fontWeight: 'bold', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{th.title}</div>
                                <div style={{ fontSize: '0.7rem', opacity: 0.6 }}>{th.rejection_count} {t.rejections}</div>
                            </div>
                        </div>
                    );
                })}
                {interrupts.length === 0 && (
                    <div style={{ flex: 1, color: 'var(--text-dim)', fontSize: '0.8rem', textAlign: 'center', padding: '1rem' }}>
                        {t.noBugs}
                    </div>
                )}
            </div>

            {/* Shared Diagnosis Area for the selected task */}
            {selectedInterruptId && (
                <div style={{ 
                    padding: '1.5rem', 
                    background: 'rgba(0,0,0,0.3)', 
                    border: '1px solid rgba(255,0,0,0.2)',
                    borderRadius: '6px',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '1.5rem',
                    animation: 'slideDown 0.3s ease-out'
                }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '0.5rem' }}>
                        <Cpu size={14} color="var(--status-error)" />
                        <span style={{ fontSize: '0.8rem', fontWeight: 'bold', color: 'var(--status-error)' }}>
                            {interrupts.find(it => it.id === selectedInterruptId)?.title} - {t.interruptReason}
                        </span>
                    </div>

                        {(() => {
                            const event = events.find(ev => ev.thread_id === selectedInterruptId && (ev.level === 'Critical' || ev.level === 'Error'));
                            if (!event || !event.content.includes('### 🧠 AI DIAGNOSIS')) return null;

                            const diagMatch = event.content.match(/### 🧠 AI DIAGNOSIS\n([\s\S]*?)(?:\n\n---|\n### 🔍 CODE_PEEK|$)/);
                            if (!diagMatch) return null;

                            const diagnosis = diagMatch[1];
                            const cause = diagnosis.match(/(?:원인:|Cause:)\s*(.*)/)?.[1] || '분석 중...';
                            const solution = diagnosis.match(/(?:해결 방법:|Solution:)\s*(.*)/)?.[1] || '지침 대기 중...';

                            return (
                                <div style={{ 
                                    marginBottom: '1.5rem', 
                                    padding: '1.2rem', 
                                    background: 'rgba(255, 152, 0, 0.1)', 
                                    border: '1px solid rgba(255, 152, 0, 0.3)',
                                    borderRadius: '8px',
                                    display: 'flex',
                                    flexDirection: 'column',
                                    gap: '0.8rem'
                                }}>
                                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: '#ff9800', fontWeight: 'bold', fontSize: '0.9rem' }}>
                                        <Cpu size={16} /> {t.aiDiagnosis || 'AI 진단 보고서'}
                                    </div>
                                    <div style={{ fontSize: '0.85rem' }}>
                                        <span style={{ color: '#ff9800', fontWeight: 'bold' }}>• 원인:</span> {cause}
                                    </div>
                                    <div style={{ fontSize: '0.85rem' }}>
                                        <span style={{ color: '#00ff88', fontWeight: 'bold' }}>• 해결 방법:</span> {solution}
                                    </div>
                                </div>
                            );
                        })()}

                        <div style={{ 
                            padding: '1.2rem', 
                            background: '#0a0a0a', 
                            border: '1px solid #333',
                            borderRadius: '6px',
                            color: '#e0e0e0',
                            fontSize: '0.75rem',
                            whiteSpace: 'pre-wrap',
                            fontFamily: '"Fira Code", "Courier New", monospace',
                            maxHeight: '300px',
                            overflowY: 'auto',
                            boxShadow: 'inset 0 0 10px rgba(0,0,0,0.5)',
                            lineHeight: '1.4'
                        }}>
                            {(() => {
                                const event = events.find(ev => ev.thread_id === selectedInterruptId && (ev.level === 'Critical' || ev.level === 'Error'));
                                if (!event) return '⚠️ [SYSTEM_STALL]: No diagnostic trace captured. Re-synthesis loop in progress...';
                                
                                const content = event.content;
                                
                                if (content.includes('[CONSENSUS_BREAKDOWN]')) {
                                    const parts = content.split('\n\n');
                                    return (
                                        <>
                                            <div style={{ color: '#ff4444', fontWeight: 'bold', borderBottom: '1px solid #333', paddingBottom: '0.5rem', marginBottom: '1rem' }}>
                                                🚨 CONSENSUS_BREAKDOWN: Senior vs Validator
                                            </div>
                                            {parts.map((p, i) => {
                                                if (p.includes('[SENIOR_REVIEW_AUDIT]')) {
                                                    return <div key={i} style={{ color: '#00e5ff', marginBottom: '1rem', borderLeft: '3px solid #00e5ff', paddingLeft: '0.8rem' }}>{p}</div>;
                                                }
                                                if (p.includes('[EXECUTION_VALIDATOR]')) {
                                                    return <div key={i} style={{ color: '#ff4444', background: 'rgba(255,68,68,0.05)', padding: '0.8rem', borderRadius: '4px' }}>{p}</div>;
                                                }
                                                if (p.includes('### 🧠 AI DIAGNOSIS')) return null; // Already shown in card
                                                return <div key={i}>{p}</div>;
                                            })}
                                        </>
                                    );
                                }

                                {(() => {
                                    const peekMatch = content.match(/### 🔍 CODE_PEEK: (.*?)\n([\s\S]*?)\n\n---/);
                                    if (!peekMatch) return null;

                                    const fileName = peekMatch[1];
                                    const snippet = peekMatch[2];

                                    return (
                                        <div style={{ marginBottom: '1.5rem' }}>
                                            <div style={{ fontSize: '0.7rem', color: 'var(--accent-primary)', marginBottom: '0.4rem', fontFamily: 'monospace' }}>
                                                🎯 THE SMOKING GUN: {fileName}
                                            </div>
                                            <div style={{ 
                                                background: '#151515', 
                                                border: '1px solid #333', 
                                                borderRadius: '4px',
                                                padding: '0.8rem',
                                                fontFamily: '"Fira Code", monospace',
                                                fontSize: '0.7rem',
                                                color: '#bbb',
                                                lineHeight: '1.2'
                                            }}>
                                                {snippet.split('\n').map((line, i) => {
                                                    const isErrorLine = line.startsWith('>>');
                                                    return (
                                                        <div key={i} style={{ 
                                                            backgroundColor: isErrorLine ? 'rgba(255,68,68,0.1)' : 'transparent',
                                                            color: isErrorLine ? '#ff4444' : 'inherit',
                                                            fontWeight: isErrorLine ? 'bold' : 'normal',
                                                            padding: '0 4px',
                                                            borderLeft: isErrorLine ? '2px solid #ff4444' : '2px solid transparent'
                                                        }}>
                                                            {line}
                                                        </div>
                                                    );
                                                })}
                                            </div>
                                        </div>
                                    );
                                })()}

                                if (content.includes('[SENIOR_REJECT]')) {
                                    const diagnosisCleaned = content.replace(/### 🧠 AI DIAGNOSIS[\s\S]*?\n\n---/, '');
                                    return (
                                        <>
                                            <div style={{ color: '#00e5ff', fontWeight: 'bold', borderBottom: '1px solid #333', paddingBottom: '0.5rem', marginBottom: '1rem' }}>
                                                🏛️ SENIOR_REJECT: Performance/Logic Violation
                                            </div>
                                            <div style={{ color: '#fff' }}>{diagnosisCleaned.replace(/🚨 \[SENIOR_REJECT\] .*\n\n/, '')}</div>
                                        </>
                                    );
                                }

                                if (content.includes('[BUILD_REJECT]')) {
                                    return (
                                        <>
                                            <div style={{ color: '#ff9800', fontWeight: 'bold', borderBottom: '1px solid #333', paddingBottom: '0.5rem', marginBottom: '1rem' }}>
                                                🛠️ BUILD_REJECT: GCC/Syntax Error
                                            </div>
                                            <div style={{ color: '#ff4444' }}>{content.replace(/🚨 \[BUILD_REJECT\] .*\n\n/, '')}</div>
                                        </>
                                    );
                                }
                                
                                return content;
                            })()}
                        </div>

                    <div style={{ display: 'flex', flexDirection: 'column' }}>
                        <label style={{ fontSize: '0.7rem', color: 'var(--text-dim)', display: 'block', marginBottom: '0.5rem' }}>{t.bossClarification}</label>
                        <textarea 
                            value={clarification}
                            onChange={(e) => setClarification(e.target.value)}
                            placeholder="명확한 기술적 지시를 입력하세요..."
                            style={{ 
                                width: '100%', 
                                height: '100px',
                                background: 'rgba(0,0,0,0.5)', 
                                border: '1px solid rgba(255,255,255,0.1)',
                                borderRadius: '4px',
                                padding: '1rem',
                                color: 'white',
                                resize: 'none',
                                fontFamily: 'inherit'
                            }}
                        />
                    </div>

                    <button 
                        className="btn-control" 
                        onClick={handleIssueClarification}
                        style={{ background: 'var(--status-error)', borderColor: 'var(--status-error)', color: 'white', alignSelf: 'flex-end' }}
                    >
                        <ShieldCheck size={16} /> {t.issueCommand}
                    </button>
                </div>
            )}
        </div>

        {/* 2. New Spec Input BELOW */}
        <div className="card" style={{ border: '1px solid var(--accent-primary)' }}>
            <h3 style={{ fontSize: '1rem', marginBottom: '1.5rem', color: 'var(--accent-primary)', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                <Cpu size={16} />
                {t.newSpecDeclaration}
            </h3>
            <form onSubmit={handleSubmitSpec} style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                <textarea 
                    value={spec}
                    onChange={(e) => setSpec(e.target.value)}
                    placeholder={t.specPlaceholder}
                    style={{ 
                        width: '100%', 
                        height: '150px', 
                        background: 'rgba(0,0,0,0.3)', 
                        border: '1px solid rgba(255,255,255,0.1)',
                        borderRadius: '4px',
                        padding: '1rem',
                        color: 'white',
                        resize: 'none',
                        fontFamily: 'inherit'
                    }}
                />
                <button type="submit" className="btn-control" style={{ alignSelf: 'flex-end', padding: '0.5rem 2rem' }}>
                    <Send size={16} /> {t.submitCommand}
                </button>
            </form>
        </div>
      </div>
    </section>
  );
};

export default BossBoard;
