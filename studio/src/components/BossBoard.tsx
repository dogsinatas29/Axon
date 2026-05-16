
import React, { useState, useEffect } from 'react';
import { ShieldCheck, CheckCircle2, Zap, ArrowRight, Gavel, Trash2, Lock } from 'lucide-react';
import { type Thread, type Event } from '../types';

interface BossBoardProps {
  threads: Thread[];
  events: Event[];
  t: any;
}

const BossBoard: React.FC<BossBoardProps> = () => {
  const [spec, setSpec] = useState('');
  const [clarification, setClarification] = useState('');
  const [semanticClosure, setSemanticClosure] = useState<any>(null);
  const [selectedRisk, setSelectedRisk] = useState<any>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [showSuccess, setShowSuccess] = useState(false);
  const [editedCode, setEditedCode] = useState('');
  const [contractModal, setContractModal] = useState<{show: boolean, content: string}>({show: false, content: ''});

  const translations: any = {
    ko_KR: {
        risk_list_title: "공정 전이 차단 목록",
        rejected_by: "반려 주체",
        failed_stage: "차단된 공정 단계",
        expected: "원천 규약 (ARCHITECTURE)",
        detected: "규약 위반 탐지 (VIOLATION)",
        sacred_contract: "확정된 규약 [성역]",
        evidence: "위반 증거 코드 (CONSTITUTIONAL EVIDENCE)",
        correction: "보스 교정 지시 (TACTICAL CORRECTION)",
        correction_placeholder: "교정할 내용이나 지침을 입력하십시오...",
        btn_seal: "승인 및 봉인 (OVERRIDE & SEAL)",
        btn_rework: "수정 후 재심",
        btn_discard: "작업 폐기",
        seal_warning: "* SEAL 결정은 헌법적 권위를 가지며, 이후 모든 공정의 기준이 됩니다.",
        idle_msg: "지휘소 대기 중",
        amendment_title: "새로운 규약 선언 (CONSTITUTION AMENDMENT)",
        amendment_placeholder: "새로운 세만틱 규약이나 아키텍처 수정을 선언하십시오...",
        btn_declare: "선언"
    },
    ja_JP: {
        risk_list_title: "工程遷移遮断リスト",
        rejected_by: "却下主体",
        failed_stage: "遮단된 工程段階",
        expected: "原典規約 (ARCHITECTURE)",
        detected: "規約違反検知 (VIOLATION)",
        sacred_contract: "確定規約 [聖域]",
        evidence: "違反証拠コード (CONSTITUTIONAL EVIDENCE)",
        correction: "社長矯正指示 (TACTICAL CORRECTION)",
        correction_placeholder: "矯正内容や指示を入力してください...",
        btn_seal: "承認及び封印 (OVERRIDE & SEAL)",
        btn_rework: "修正後再審",
        btn_discard: "作業廃棄",
        seal_warning: "* SEAL決定は憲法的な権威を持ち、以後の全工程の基準となります。",
        idle_msg: "指令所待機中",
        amendment_title: "新しい規約宣言 (CONSTITUTION AMENDMENT)",
        amendment_placeholder: "新しいセマンティック規約やアーキテクチャ修正を宣言してください...",
        btn_declare: "宣言"
    },
    en_US: {
        risk_list_title: "STATE TRANSITION BLOCKED",
        rejected_by: "REJECTED BY",
        failed_stage: "FAILED STAGE",
        expected: "EXPECTED (ARCHITECTURE)",
        detected: "DETECTED (VIOLATION)",
        sacred_contract: "SACRED CONTRACT [LOCKED]",
        evidence: "CONSTITUTIONAL EVIDENCE",
        correction: "TACTICAL CORRECTION",
        correction_placeholder: "Enter corrections or instructions...",
        btn_seal: "OVERRIDE & SEAL",
        btn_rework: "REWORK",
        btn_discard: "DISCARD",
        seal_warning: "* SEAL decisions have constitutional authority and define future stages.",
        idle_msg: "GOVERNANCE IDLE",
        amendment_title: "CONSTITUTION AMENDMENT",
        amendment_placeholder: "Declare new semantic rules or architecture updates...",
        btn_declare: "DECLARE"
    }
  };

  const currentT = translations[semanticClosure?.locale] || translations.ko_KR;

  const triggerSuccess = () => {
    setShowSuccess(true);
    setTimeout(() => setShowSuccess(false), 2500);
  };

  useEffect(() => {
    if (selectedRisk && selectedRisk.full_code) {
        setEditedCode(selectedRisk.full_code);
    } else {
        setEditedCode('');
    }
  }, [selectedRisk]);

  useEffect(() => {
    const fetchRisks = async () => {
      try {
        const response = await fetch(`/api/semantics/risks`);
        if (response.ok) {
          const data = await response.json();
          setSemanticClosure(data);
          if (selectedRisk) {
            const freshRisk = data.risks.find((r: any) => r.risk_id === selectedRisk.risk_id);
            if (freshRisk) setSelectedRisk(freshRisk);
          }
        }
      } catch (err) {
        console.error('Failed to fetch risks:', err);
      }
    };
    const interval = setInterval(fetchRisks, 2000);
    return () => clearInterval(interval);
  }, [selectedRisk]);

  const handleSemanticDecision = async (action: string) => {
    if (!selectedRisk || isSubmitting) return;
    setIsSubmitting(true);
    try {
      const response = await fetch(`/api/semantics/decide`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          risk_id: selectedRisk.risk_id,
          action,
          comment: clarification || 'Sovereign decision finalized by Boss',
          code: action === 'SEAL' ? editedCode : null
        }),
      });
      if (response.ok) {
        triggerSuccess();
        setSelectedRisk(null);
        setClarification('');
      }
    } catch (err) {
      console.error('Decision failed:', err);
    } finally {
      setIsSubmitting(false);
    }
  };

  const PipelineStage = ({ label, active, failed }: { label: string, active: boolean, failed?: boolean }) => (
    <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', opacity: active ? 1 : 0.3 }}>
        <div style={{ 
            width: '12px', height: '12px', borderRadius: '50%', 
            background: failed ? '#ff4444' : (active ? '#00ffaa' : '#888'),
            boxShadow: active ? `0 0 10px ${failed ? '#ff4444' : '#00ffaa'}` : 'none'
        }} />
        <span style={{ fontSize: '0.7rem', fontWeight: 'bold', color: failed ? '#ff4444' : '#fff' }}>{label}</span>
    </div>
  );

  return (
    <section className="panel" style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', background: '#050505', height: '100%', minHeight: 0 }}>
      {showSuccess && <div className="success-toast"><CheckCircle2 size={32} /> 판결이 공정에 즉각 반영되었습니다!</div>}

      {/* SACRED CONTRACT MODAL */}
      {contractModal.show && (
          <div style={{ position: 'fixed', top: 0, left: 0, width: '100vw', height: '100vh', background: 'rgba(0,0,0,0.85)', zIndex: 1000, display: 'flex', alignItems: 'center', justifyContent: 'center', backdropFilter: 'blur(10px)' }}>
              <div style={{ width: '800px', maxWidth: '90%', background: '#111', border: '1px solid var(--accent-primary)', borderRadius: '24px', padding: '3rem', position: 'relative', boxShadow: '0 0 50px rgba(0, 242, 255, 0.2)' }}>
                  <button onClick={() => setContractModal({show: false, content: ''})} style={{ position: 'absolute', top: '1.5rem', right: '1.5rem', background: 'transparent', border: 'none', color: '#888', cursor: 'pointer' }}>
                      <Trash2 size={24} />
                  </button>
                  <div style={{ fontSize: '0.8rem', color: 'var(--accent-primary)', fontWeight: 'bold', marginBottom: '1rem', textTransform: 'uppercase', letterSpacing: '0.2rem' }}>[SACRED DESIGN PROTOCOL]</div>
                  <h3 style={{ fontSize: '1.8rem', fontWeight: '900', color: '#fff', marginBottom: '2rem' }}>Architectural Requirements</h3>
                  <div style={{ background: '#000', padding: '2rem', borderRadius: '16px', color: '#00ffaa', fontFamily: 'monospace', fontSize: '1.1rem', lineHeight: '1.6', overflowY: 'auto', maxHeight: '400px', border: '1px solid rgba(255,255,255,0.05)' }}>
                      {contractModal.content || 'No detailed spec found for this contract.'}
                  </div>
                  <div style={{ marginTop: '2rem', textAlign: 'right' }}>
                      <button onClick={() => setContractModal({show: false, content: ''})} className="btn-control" style={{ background: 'var(--accent-primary)', color: '#000', padding: '1rem 3rem', fontWeight: 'bold' }}>ACKNOWLEDGE</button>
                  </div>
              </div>
          </div>
      )}

      <div style={{ background: 'linear-gradient(90deg, #ff4444, #800000)', color: 'white', padding: '1rem 2rem', display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexShrink: 0 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <ShieldCheck size={28} />
          <span style={{ fontSize: '1.4rem', fontWeight: '900', letterSpacing: '0.1rem' }}>SEMANTIC GOVERNANCE CONSOLE</span>
        </div>
        <div style={{ display: 'flex', gap: '1.5rem', alignItems: 'center' }}>
            <PipelineStage label="SPEC" active={true} />
            <ArrowRight size={14} opacity={0.3} />
            <PipelineStage label="ARCH" active={true} />
            <ArrowRight size={14} opacity={0.3} />
            <PipelineStage label="CONTRACT" active={true} failed={!!selectedRisk} />
            <ArrowRight size={14} opacity={0.3} />
            <PipelineStage label="BLOCK" active={!!selectedRisk} failed={!!selectedRisk} />
        </div>
      </div>
      
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden', minHeight: 0 }}>
        {/* Risk Sidebar */}
        <div style={{ width: '380px', borderRight: '1px solid rgba(255,255,255,0.05)', background: 'rgba(255,255,255,0.02)', overflowY: 'auto', padding: '1.5rem', flexShrink: 0 }}>
            <div style={{ fontSize: '0.7rem', color: 'rgba(255,255,255,0.4)', fontWeight: 'bold', marginBottom: '1.5rem', textTransform: 'uppercase', letterSpacing: '0.1rem', display: 'flex', justifyContent: 'space-between' }}>
                <span>{currentT.risk_list_title}</span>
                <span>{semanticClosure?.risks?.length || 0}</span>
            </div>
            {semanticClosure?.risks?.sort((a: any, b: any) => {
                const getP = (s: string) => s?.includes('Phase 1') ? 1 : (s?.includes('Phase 2') ? 2 : 3);
                return getP(a.failed_stage) - getP(b.failed_stage);
            }).map((risk: any) => (
                <div key={risk.risk_id} onClick={() => setSelectedRisk(risk)} style={{ 
                    cursor: 'pointer', padding: '1.2rem', marginBottom: '1rem', borderRadius: '12px',
                    border: selectedRisk?.risk_id === risk.risk_id ? `2px solid #ff4444` : '1px solid rgba(255,255,255,0.05)',
                    background: selectedRisk?.risk_id === risk.risk_id ? 'rgba(255, 68, 68, 0.15)' : 'rgba(255,255,255,0.02)',
                    boxShadow: selectedRisk?.risk_id === risk.risk_id ? '0 0 20px rgba(255, 68, 68, 0.2)' : 'none',
                    transition: 'all 0.2s'
                }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.6rem' }}>
                        <div style={{ fontSize: '0.6rem', color: 'var(--accent-primary)', fontWeight: 'bold', background: 'rgba(0, 242, 255, 0.1)', padding: '2px 6px', borderRadius: '4px' }}>
                            {risk.failed_stage?.split(' ')[0] || 'TASK'} #{risk.risk_id?.slice(-4).toUpperCase()}
                        </div>
                        <div style={{ fontSize: '0.6rem', color: 'rgba(255,255,255,0.3)' }}>{risk.timestamp || 'Just now'}</div>
                    </div>
                    
                    <div style={{ fontSize: '1rem', fontWeight: 'bold', color: '#fff', marginBottom: '0.4rem' }}>{risk.target || risk.component}</div>
                    
                    <div style={{ fontSize: '0.7rem', color: '#ff4444', fontWeight: 'bold', marginBottom: '0.8rem' }}>
                        {risk.failed_stage || 'BLOCKER DETECTED'}
                    </div>

                    {/* Reject Matrix for BossBoard */}
                    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.4rem', background: 'rgba(0,0,0,0.3)', padding: '0.6rem', borderRadius: '8px' }}>
                        <div style={{ fontSize: '0.55rem', color: 'rgba(255,255,255,0.4)', display: 'flex', justifyContent: 'space-between' }}>
                            <span>VAL</span>
                            <span style={{ color: risk.validator_rejects > 0 ? '#ff4444' : '#666' }}>{risk.validator_rejects || 0}</span>
                        </div>
                        <div style={{ fontSize: '0.55rem', color: 'rgba(255,255,255,0.4)', display: 'flex', justifyContent: 'space-between' }}>
                            <span>SNR</span>
                            <span style={{ color: risk.senior_rejects > 0 ? '#ff4444' : '#666' }}>{risk.senior_rejects || 0}</span>
                        </div>
                        <div style={{ fontSize: '0.55rem', color: 'rgba(255,255,255,0.4)', display: 'flex', justifyContent: 'space-between' }}>
                            <span>ARC</span>
                            <span style={{ color: risk.architect_rejects > 0 ? '#ff4444' : '#666' }}>{risk.architect_rejects || 0}</span>
                        </div>
                        <div style={{ fontSize: '0.55rem', color: 'rgba(255,255,255,0.4)', display: 'flex', justifyContent: 'space-between' }}>
                            <span>CMK</span>
                            <span style={{ color: risk.cmake_rejects > 0 ? '#ff4444' : '#666' }}>{risk.cmake_rejects || 0}</span>
                        </div>
                    </div>
                </div>
            ))}
        </div>

        {/* Console View - THE TACTICAL DESK */}
        <div style={{ flex: 1, display: 'grid', gridTemplateRows: '1fr auto', background: '#080808', overflow: 'hidden', minHeight: 0 }}>
            {selectedRisk ? (
                <>
                  {/* 1. Scrollable Report Area (Diagnostics & Code) */}
                  <div style={{ overflowY: 'auto', padding: '2.5rem', display: 'flex', flexDirection: 'column', gap: '2rem' }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                          <div>
                              <div style={{ fontSize: '0.8rem', color: '#ff4444', fontWeight: 'bold', marginBottom: '0.5rem' }}>[{currentT.rejected_by}: {selectedRisk.actor || '감찰관'}]</div>
                              <h2 style={{ fontSize: '2.5rem', fontWeight: '900', color: '#fff', letterSpacing: '-0.05rem' }}>{selectedRisk.target || selectedRisk.component}</h2>
                          </div>
                          <div style={{ textAlign: 'right' }}>
                              <div style={{ fontSize: '0.7rem', color: 'rgba(255,255,255,0.4)', marginBottom: '0.5rem' }}>{currentT.failed_stage}</div>
                              <div style={{ color: '#ff4444', fontWeight: 'bold', fontSize: '1.1rem' }}>{selectedRisk.failed_stage}</div>
                          </div>
                      </div>

                      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.5rem' }}>
                          <div style={{ background: 'rgba(255,255,255,0.03)', padding: '1.5rem', borderRadius: '12px', border: '1px solid rgba(255,255,255,0.05)' }}>
                              <div style={{ fontSize: '0.7rem', color: '#00ffaa', fontWeight: 'bold', marginBottom: '1rem' }}>{currentT.expected}</div>
                              <div style={{ fontSize: '1rem', color: '#00ffaa', fontFamily: 'monospace' }}>{selectedRisk.expected || '설계 규약을 찾을 수 없음'}</div>
                          </div>
                          <div style={{ background: 'rgba(255, 68, 68, 0.05)', padding: '1.5rem', borderRadius: '12px', border: '1px solid rgba(255, 68, 68, 0.1)' }}>
                              <div style={{ fontSize: '0.7rem', color: '#ff4444', fontWeight: 'bold', marginBottom: '1rem' }}>{currentT.detected}</div>
                              <div style={{ fontSize: '1rem', color: '#ff4444', fontFamily: 'monospace' }}>{selectedRisk.detected || '위반 사항 식별됨'}</div>
                          </div>
                      </div>

                      {/* 1.1.5 VIOLATION ANALYSIS LOG */}
                      <div style={{ padding: '1.2rem', background: 'rgba(255, 68, 68, 0.08)', borderRadius: '12px', borderLeft: '4px solid #ff4444' }}>
                          <div style={{ fontSize: '0.7rem', color: '#ff4444', fontWeight: 'bold', marginBottom: '0.5rem', textTransform: 'uppercase' }}>VIOLATION TRACE</div>
                          <div style={{ fontSize: '0.85rem', color: '#ff8888', fontStyle: 'italic', lineHeight: '1.4' }}>
                              {selectedRisk.analysis || '분석 중: 함수 호출 시그니처 미매칭 및 선언되지 않은 외부 헤더 사용 감지됨.'}
                          </div>
                      </div>

                      <div style={{ display: 'grid', gridTemplateColumns: '1fr 280px', gap: '1.5rem' }}>
                          <div style={{ position: 'relative', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '16px', background: '#000', overflow: 'hidden' }}>
                              <div style={{ background: 'rgba(255,255,255,0.03)', padding: '1rem 1.5rem', fontSize: '0.75rem', fontWeight: 'bold', borderBottom: '1px solid rgba(255,255,255,0.05)', color: 'rgba(255,255,255,0.5)', display: 'flex', justifyContent: 'space-between' }}>
                                  <span>{currentT.evidence}</span>
                                  <span style={{ color: 'var(--accent-primary)', fontSize: '0.6rem' }}>[BOSS DIRECT EDIT ENABLED]</span>
                              </div>
                              <div style={{ padding: '0' }}>
                                  <textarea 
                                    value={editedCode} 
                                    onChange={(e) => setEditedCode(e.target.value)}
                                    spellCheck={false}
                                    style={{ 
                                        width: '100%', height: '350px', background: '#000', color: '#00ffaa', 
                                        border: 'none', padding: '1.5rem', fontFamily: 'monospace', fontSize: '1rem',
                                        lineHeight: '1.5', resize: 'none', outline: 'none'
                                    }}
                                  />
                              </div>
                          </div>
                          
                          {/* SACRED CONTRACT POPUP TRIGGER */}
                          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                              <div 
                                onClick={() => setContractModal({show: true, content: selectedRisk.expected || 'Design Spec Not Found'})}
                                style={{ 
                                  cursor: 'pointer',
                                  padding: '1.2rem', 
                                  borderRadius: '12px', 
                                  background: 'rgba(0, 242, 255, 0.05)', 
                                  border: '1px solid rgba(0, 242, 255, 0.2)',
                                  transition: 'all 0.2s'
                                }}
                                onMouseOver={(e) => e.currentTarget.style.background = 'rgba(0, 242, 255, 0.1)'}
                                onMouseOut={(e) => e.currentTarget.style.background = 'rgba(0, 242, 255, 0.05)'}
                              >
                                  <div style={{ fontSize: '0.6rem', color: 'var(--accent-primary)', fontWeight: '900', marginBottom: '0.6rem', display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
                                      <ShieldCheck size={12} /> SACRED CONTRACT [LOCKED]
                                  </div>
                                  <div style={{ fontSize: '0.8rem', color: '#fff', fontWeight: 'bold' }}>{selectedRisk.expected || 'Design Spec'}</div>
                                  <div style={{ fontSize: '0.6rem', color: 'rgba(255,255,255,0.4)', marginTop: '0.6rem' }}>CLICK TO VIEW FULL PROTOCOL</div>
                              </div>
                              
                              <div style={{ padding: '1rem', borderRadius: '12px', background: 'rgba(255,255,255,0.02)', border: '1px solid rgba(255,255,255,0.05)' }}>
                                  <div style={{ fontSize: '0.6rem', color: 'rgba(255,255,255,0.4)', fontWeight: 'bold', marginBottom: '0.8rem' }}>HIGHLIGHTED ERRORS</div>
                                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.6rem' }}>
                                      <div style={{ fontSize: '0.75rem', color: '#ff4444', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                          <Zap size={10} /> sqlite3 usage [L12]
                                      </div>
                                      <div style={{ fontSize: '0.75rem', color: '#ff4444', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                          <Zap size={10} /> Missing Header [L1]
                                      </div>
                                  </div>
                              </div>
                          </div>
                      </div>
                  </div>

                  {/* 2. Fixed Tactical Command Area (Actions & Amendments) */}
                  <div style={{ padding: '1.5rem 2.5rem', borderTop: '2px solid rgba(255, 255, 255, 0.1)', background: '#0a0a0f', boxShadow: '0 -10px 30px rgba(0,0,0,0.5)', zIndex: 100, display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
                      {/* Tactical Correction + Buttons */}
                      <div>
                          <div style={{ display: 'flex', gap: '1rem', marginBottom: '1rem' }}>
                              <div style={{ flex: 1 }}>
                                  <div style={{ fontSize: '0.7rem', color: 'var(--accent-primary)', fontWeight: 'bold', marginBottom: '0.4rem' }}>{currentT.correction}</div>
                                  <input value={clarification} onChange={(e) => setClarification(e.target.value)} placeholder={currentT.correction_placeholder}
                                      style={{ width: '100%', background: '#000', border: '1px solid rgba(0, 242, 255, 0.3)', borderRadius: '8px', padding: '0.8rem', color: '#fff' }} />
                              </div>
                          </div>
                          <div style={{ display: 'flex', gap: '1rem' }}>
                              <button onClick={() => handleSemanticDecision('SEAL')} style={{ flex: 1.5, background: '#00ffaa', color: '#000', fontWeight: 'bold', padding: '0.8rem', borderRadius: '8px', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem' }}>
                                  <Gavel size={18} /> {currentT.btn_seal}
                              </button>
                              <button onClick={() => handleSemanticDecision('REWORK')} style={{ flex: 2, background: 'var(--accent-primary)', color: '#000', fontWeight: 'bold', padding: '0.8rem', borderRadius: '8px', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem' }}>
                                  <Zap size={18} /> {currentT.btn_rework}
                              </button>
                              <button onClick={() => handleSemanticDecision('STOP')} style={{ flex: 1, background: 'rgba(255,255,255,0.05)', color: '#ff4444', border: '1px solid rgba(255,68,68,0.2)', padding: '0.8rem', borderRadius: '8px', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem' }}>
                                  <Trash2 size={18} /> {currentT.btn_discard}
                              </button>
                          </div>
                          <div style={{ marginTop: '0.8rem', fontSize: '0.6rem', color: 'rgba(255,255,255,0.3)', textAlign: 'center' }}>
                              {currentT.seal_warning}
                          </div>
                      </div>

                      {/* Constitution Amendment Section */}
                      <div style={{ padding: '1.2rem', borderRadius: '12px', background: 'rgba(0, 242, 255, 0.02)', border: '1px dashed rgba(0, 242, 255, 0.2)' }}>
                          <h3 style={{ fontSize: '0.7rem', color: 'var(--accent-primary)', fontWeight: 'bold', marginBottom: '0.8rem', display: 'flex', alignItems: 'center', gap: '0.6rem', textTransform: 'uppercase', letterSpacing: '0.1rem' }}>
                              <Lock size={14} /> {currentT.amendment_title}
                          </h3>
                          <div style={{ display: 'flex', gap: '0.8rem' }}>
                              <input value={spec} onChange={(e) => setSpec(e.target.value)} placeholder={currentT.amendment_placeholder}
                                  style={{ flex: 1, background: 'rgba(0,0,0,0.5)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', padding: '0.7rem', color: 'white', fontSize: '0.85rem' }} />
                              <button onClick={async () => {
                                  if (!spec.trim()) return;
                                  setIsSubmitting(true);
                                  try {
                                      const response = await fetch(`/api/specs`, {
                                          method: 'POST', headers: { 'Content-Type': 'application/json' },
                                          body: JSON.stringify({ content: spec }),
                                      });
                                      if (response.ok) { triggerSuccess(); setSpec(''); }
                                  } finally { setIsSubmitting(false); }
                              }} className="btn-control" style={{ background: 'rgba(0, 242, 255, 0.1)', color: 'var(--accent-primary)', border: '1px solid var(--accent-primary)', padding: '0 1.5rem', fontSize: '0.85rem' }}>{currentT.btn_declare}</button>
                          </div>
                      </div>
                  </div>
                </>
            ) : (
                <div style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', opacity: 0.1 }}>
                    <ShieldCheck size={120} />
                    <div style={{ fontSize: '1.5rem', fontWeight: '900', marginTop: '1rem' }}>{currentT.idle_msg}</div>
                </div>
            )}
        </div>
      </div>
    </section>
  );
};

export default BossBoard;
