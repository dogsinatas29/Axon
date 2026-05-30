import React from 'react';
import type { Thread, ThreadStatus } from '../types';
import { 
  CheckCircle, 
  Clock, 
  AlertCircle, 
  ShieldCheck, 
  Zap
} from 'lucide-react';

interface ThreadCardProps {
  thread: Thread;
  onClick: (id: string) => void;
  t: any;
}

const parseTaskKindStr = (taskKind: any): string | undefined => {
  if (!taskKind) return undefined;
  if (typeof taskKind === 'string') {
    return taskKind;
  }
  if (typeof taskKind === 'object') {
    const values = Object.values(taskKind);
    if (values.length > 0) {
      return values[0] as string;
    }
  }
  return undefined;
};

const ThreadCard: React.FC<ThreadCardProps> = ({ thread, onClick, t }) => {
  const { id, title, status, updated_at, rejection_count } = thread;

  const getStatusConfig = (status: ThreadStatus) => {
    switch (status) {
      case 'Approved':
        return { color: '#00e5ff', icon: <ShieldCheck size={18} />, glow: '0 0 20px rgba(0, 229, 255, 0.4)' };
      case 'Completed':
        return { color: '#00ff88', icon: <CheckCircle size={18} />, glow: '0 0 20px rgba(0, 255, 136, 0.4)' };
      case 'Rejected':
        return { color: '#ff4444', icon: <AlertCircle size={18} />, glow: '0 0 20px rgba(255, 68, 68, 0.4)' };
      case 'Working':
        return { color: '#ffd700', icon: <Zap size={18} className="animate-pulse" />, glow: '0 0 15px rgba(255, 215, 0, 0.3)' };
      case 'AwaitDependency':
        return { color: '#38bdf8', icon: <Clock size={18} className="animate-spin" style={{ animationDuration: '3s' }} />, glow: '0 0 20px rgba(56, 189, 248, 0.3)' };
      default:
        return { color: '#888', icon: <Clock size={18} />, glow: 'none' };
    }
  };

  const config = getStatusConfig(status);
  const kindStr = parseTaskKindStr(thread.task_kind);

  return (
    <div 
      onClick={() => onClick(id)}
      style={{
        backgroundColor: '#111',
        borderRadius: '16px',
        padding: '16px 20px',
        cursor: 'pointer',
        transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
        border: rejection_count && rejection_count >= 3 ? '3px solid #ff0000' : `1px solid ${config.color}`,
        boxShadow: rejection_count && rejection_count >= 3 ? '0 0 25px rgba(255, 0, 0, 0.3)' : config.glow,
        position: 'relative',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
        gap: '8px'
      }}
    >
      {/* Header: ID & Status */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span style={{ fontSize: '0.7rem', color: 'rgba(255,255,255,0.3)', fontFamily: 'monospace', letterSpacing: '0.05rem' }}>#{id.split('_').pop()?.toUpperCase()}</span>
        <div style={{ 
          display: 'flex', 
          alignItems: 'center', 
          gap: '8px', 
          color: config.color,
          fontSize: '0.75rem',
          fontWeight: '900',
          textTransform: 'uppercase',
          padding: '4px 10px',
          background: 'rgba(255,255,255,0.03)',
          borderRadius: '20px',
          border: `1px solid ${config.color}33`
        }}>
          {config.icon}
          {status === 'Working' ? 'WORKING' : status === 'AwaitDependency' ? 'AWAIT DEP' : status.toUpperCase()}
        </div>
      </div>

      {/* Main Info: Phase & Title */}
      <div style={{ display: 'flex', gap: '12px', alignItems: 'flex-start' }}>
        {thread.task_kind && (
          <div style={{ 
            fontSize: '0.65rem', 
            backgroundColor: 'rgba(0, 242, 255, 0.1)', 
            color: '#00f2ff', 
            padding: '6px 10px', 
            borderRadius: '8px',
            border: '1px solid rgba(0, 242, 255, 0.3)',
            fontFamily: 'Orbitron',
            textAlign: 'center',
            minWidth: '70px'
          }}>
            <div style={{ opacity: 0.6, fontSize: '0.5rem', marginBottom: '2px' }}>PHASE</div>
            <div style={{ fontSize: '1.1rem', fontWeight: 'bold' }}>
              {kindStr === 'HeaderDecl' || kindStr === 'ModuleDecl' ? '01' : kindStr === 'SourceImpl' || kindStr === 'ModuleImpl' ? '02' : '03'}
            </div>
          </div>
        )}
        <h3 style={{ 
          margin: 0, 
          fontSize: '1.2rem', 
          color: status === 'Working' ? '#ffd700' : '#fff',
          fontWeight: '800',
          lineHeight: '1.3',
          flex: 1
        }}>
          {title}
        </h3>
      </div>

      {/* Timestamp */}
      <div style={{ fontSize: '0.7rem', color: 'rgba(255,255,255,0.4)', display: 'flex', alignItems: 'center', gap: '6px' }}>
        <Clock size={12} />
        {t.updated} {updated_at}
      </div>

      {/* Granular Reject Matrix */}
      <div style={{ 
        marginTop: '4px', 
        display: 'flex',
        flexDirection: 'column',
        gap: '6px',
        padding: '10px 14px',
        background: 'rgba(0,0,0,0.4)',
        borderRadius: '12px',
        border: '1px solid rgba(255,255,255,0.06)'
      }}>
        {/* Validator */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: '0.75rem', fontWeight: '800', color: thread.validator_rejections && thread.validator_rejections > 0 ? '#ff4444' : '#888' }}>
            Validator
          </span>
          <span style={{ 
            fontSize: '0.7rem', 
            fontWeight: '900', 
            color: thread.validator_rejections && thread.validator_rejections > 0 ? '#ff4444' : '#555',
            display: 'flex',
            alignItems: 'center',
            gap: '4px'
          }}>
            {thread.validator_rejections && thread.validator_rejections > 0 ? (
              <>{t.alertPrefix} <span style={{ textDecoration: 'underline' }}>{thread.validator_rejections}{t.rejections}</span></>
            ) : (
              t.zeroCount
            )}
          </span>
        </div>

        {/* SENIOR */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: '0.75rem', fontWeight: '800', color: thread.senior_rejections && thread.senior_rejections > 0 ? '#ff4444' : '#888' }}>
            SENIOR
          </span>
          <span style={{ 
            fontSize: '0.7rem', 
            fontWeight: '900', 
            color: thread.senior_rejections && thread.senior_rejections > 0 ? '#ff4444' : '#555',
            display: 'flex',
            alignItems: 'center',
            gap: '4px'
          }}>
            {thread.senior_rejections && thread.senior_rejections > 0 ? (
              <>{t.alertPrefix} <span style={{ textDecoration: 'underline' }}>{thread.senior_rejections}{t.rejections}</span></>
            ) : (
              t.zeroCount
            )}
          </span>
        </div>

        {/* Architecture */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: '0.75rem', fontWeight: '800', color: thread.architecture_rejections && thread.architecture_rejections > 0 ? '#ff4444' : '#888' }}>
            Architecture
          </span>
          <span style={{ 
            fontSize: '0.7rem', 
            fontWeight: '900', 
            color: thread.architecture_rejections && thread.architecture_rejections > 0 ? '#ff4444' : '#555',
            display: 'flex',
            alignItems: 'center',
            gap: '4px'
          }}>
            {thread.architecture_rejections && thread.architecture_rejections > 0 ? (
              <>{t.alertPrefix} <span style={{ textDecoration: 'underline' }}>{thread.architecture_rejections}{t.rejections}</span></>
            ) : (
              t.zeroCount
            )}
          </span>
        </div>

        {/* CMake / Cargo */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: '0.75rem', fontWeight: '800', color: thread.cargo_rejections && thread.cargo_rejections > 0 ? '#ff4444' : '#888' }}>
            CMake / Cargo
          </span>
          <span style={{ 
            fontSize: '0.7rem', 
            fontWeight: '900', 
            color: thread.cargo_rejections && thread.cargo_rejections > 0 ? '#ff4444' : '#555',
            display: 'flex',
            alignItems: 'center',
            gap: '4px'
          }}>
            {thread.cargo_rejections && thread.cargo_rejections > 0 ? (
              <>{t.alertPrefix} <span style={{ textDecoration: 'underline' }}>{thread.cargo_rejections}{t.rejections}</span></>
            ) : (
              t.zeroCount
            )}
          </span>
        </div>

        {/* LSP */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: '0.75rem', fontWeight: '800', color: thread.lsp_rejections && thread.lsp_rejections > 0 ? '#ff4444' : '#888' }}>
            LSP
          </span>
          <span style={{ 
            fontSize: '0.7rem', 
            fontWeight: '900', 
            color: thread.lsp_rejections && thread.lsp_rejections > 0 ? '#ff4444' : '#555',
            display: 'flex',
            alignItems: 'center',
            gap: '4px'
          }}>
            {thread.lsp_rejections && thread.lsp_rejections > 0 ? (
              <>{t.alertPrefix} <span style={{ textDecoration: 'underline' }}>{thread.lsp_rejections}{t.rejections}</span></>
            ) : (
              t.zeroCount
            )}
          </span>
        </div>

        {/* 사장 복구 (Sovereign Interventions) */}
        <div style={{ 
          display: 'flex', 
          justifyContent: 'space-between', 
          alignItems: 'center',
          marginTop: '6px',
          paddingTop: '6px',
          borderTop: '1px dashed rgba(255,255,255,0.08)'
        }}>
          <span style={{ 
            fontSize: '0.75rem', 
            fontWeight: '900', 
            color: thread.boss_interventions && thread.boss_interventions > 0 ? '#d8b4fe' : '#888',
            display: 'flex',
            alignItems: 'center',
            gap: '4px'
          }}>
            {t.bossIntervention}
          </span>
          <span style={{ 
            fontSize: '0.7rem', 
            fontWeight: '900', 
            color: thread.boss_interventions && thread.boss_interventions > 0 ? '#c084fc' : '#555',
            display: 'flex',
            alignItems: 'center',
            gap: '4px'
          }}>
            {thread.boss_interventions && thread.boss_interventions > 0 ? (
              <span style={{
                background: 'rgba(168, 85, 247, 0.2)',
                color: '#e9d5ff',
                padding: '2px 8px',
                borderRadius: '12px',
                border: '1px solid rgba(168, 85, 247, 0.4)',
                boxShadow: '0 0 8px rgba(168, 85, 247, 0.3)'
              }}>
                {thread.boss_interventions}{t.interventionsRestored}
              </span>
            ) : (
              t.zeroCount
            )}
          </span>
        </div>
      </div>
      
      {/* Orchestration Freeze Alert Badge */}
      {status === 'AwaitDependency' && (
        <div style={{ 
          fontSize: '0.75rem', 
          color: '#38bdf8', 
          backgroundColor: 'rgba(56, 189, 248, 0.1)',
          padding: '10px',
          borderRadius: '8px',
          border: '1px solid rgba(56, 189, 248, 0.4)',
          textAlign: 'center',
          fontWeight: '900',
          letterSpacing: '0.02rem',
          display: 'flex',
          flexDirection: 'column',
          gap: '4px'
        }}>
          <div>❄️ AWAITING DEPENDENCIES - SYSTEM FROZEN</div>
          <div style={{ fontSize: '0.65rem', opacity: 0.8, fontWeight: 'normal', textDecoration: 'underline' }}>
            {thread.reason || "unresolved dependencies / awaiting predecessor task"}
          </div>
        </div>
      )}

      {/* Total Warning Label */}
      {rejection_count !== undefined && rejection_count >= 3 && (
        <div style={{ 
          fontSize: '0.7rem', 
          color: '#ff4444', 
          backgroundColor: 'rgba(255, 0, 0, 0.1)',
          padding: '8px',
          borderRadius: '8px',
          border: '1px solid #ff0000',
          textAlign: 'center',
          fontWeight: '900',
          letterSpacing: '0.05rem',
          animation: 'pulse-red 1.5s infinite'
        }}>
          ⚠️ {rejection_count} TOTAL REJECTIONS - INTERVENTION REQUIRED
        </div>
      )}
      
      {status === 'Working' && (
        <div style={{
          position: 'absolute',
          bottom: 0,
          left: 0,
          height: '3px',
          width: '100%',
          background: 'linear-gradient(90deg, transparent, #ffd700, transparent)',
          animation: 'scan 2s linear infinite'
        }} />
      )}
    </div>
  );
};

export default ThreadCard;
