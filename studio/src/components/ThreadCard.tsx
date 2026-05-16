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
      default:
        return { color: '#888', icon: <Clock size={18} />, glow: 'none' };
    }
  };

  const config = getStatusConfig(status);

  return (
    <div 
      onClick={() => onClick(id)}
      style={{
        backgroundColor: '#111',
        borderRadius: '16px',
        padding: '24px',
        cursor: 'pointer',
        transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
        border: rejection_count && rejection_count >= 3 ? '4px solid #ff0000' : `1px solid ${config.color}`,
        boxShadow: rejection_count && rejection_count >= 3 ? '0 0 30px rgba(255, 0, 0, 0.4)' : config.glow,
        position: 'relative',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
        gap: '12px'
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
          {status === 'Working' ? 'WORKING' : status.toUpperCase()}
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
              {thread.task_kind === 'HeaderDecl' ? '01' : thread.task_kind === 'SourceImpl' ? '02' : '03'}
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
        marginTop: '8px', 
        display: 'grid', 
        gridTemplateColumns: '1fr 1fr', 
        gap: '8px',
        padding: '12px',
        background: 'rgba(0,0,0,0.3)',
        borderRadius: '12px',
        border: '1px solid rgba(255,255,255,0.05)'
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: '0.6rem', color: 'rgba(255,255,255,0.4)' }}>VALIDATOR</span>
          <span style={{ fontSize: '0.75rem', fontWeight: 'bold', color: (thread as any).validator_rejections > 0 ? '#ff4444' : '#666' }}>
            {(thread as any).validator_rejections || 0}
          </span>
        </div>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: '0.6rem', color: 'rgba(255,255,255,0.4)' }}>SENIOR</span>
          <span style={{ fontSize: '0.75rem', fontWeight: 'bold', color: (thread as any).senior_rejections > 0 ? '#ff4444' : '#666' }}>
            {(thread as any).senior_rejections || 0}
          </span>
        </div>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: '0.6rem', color: 'rgba(255,255,255,0.4)' }}>ARCHITECT</span>
          <span style={{ fontSize: '0.75rem', fontWeight: 'bold', color: (thread as any).architect_rejections > 0 ? '#ff4444' : '#666' }}>
            {(thread as any).architect_rejections || 0}
          </span>
        </div>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: '0.6rem', color: 'rgba(255,255,255,0.4)' }}>CMAKE</span>
          <span style={{ fontSize: '0.75rem', fontWeight: 'bold', color: (thread as any).cmake_rejections > 0 ? '#ff4444' : '#666' }}>
            {(thread as any).cmake_rejections || 0}
          </span>
        </div>
      </div>
      
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
