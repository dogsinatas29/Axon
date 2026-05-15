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
        backgroundColor: '#1a1a1a',
        borderRadius: '12px',
        padding: '20px',
        cursor: 'pointer',
        transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
        border: rejection_count && rejection_count >= 3 ? '3px solid #ff0000' : `1px solid ${config.color}`,
        boxShadow: rejection_count && rejection_count >= 3 ? '0 0 20px rgba(255, 0, 0, 0.6)' : config.glow,
        position: 'relative',
        overflow: 'hidden',
        marginBottom: '16px'
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '12px' }}>
        <span style={{ fontSize: '10px', color: '#666', fontFamily: 'monospace' }}>{id}</span>
        <div style={{ 
          display: 'flex', 
          alignItems: 'center', 
          gap: '6px', 
          color: config.color,
          fontSize: '12px',
          fontWeight: 'bold',
          textTransform: 'uppercase',
          letterSpacing: '1px'
        }}>
          {config.icon}
          {status}
        </div>
      </div>

      <h3 style={{ 
        margin: '0 0 12px 0', 
        fontSize: '18px', 
        color: status === 'Working' ? '#ffd700' : '#fff',
        fontWeight: '600',
        display: 'flex',
        alignItems: 'center',
        gap: '10px'
      }}>
        {thread.task_kind && (
          <span style={{ 
            fontSize: '10px', 
            backgroundColor: 'rgba(0, 229, 255, 0.1)', 
            color: '#00e5ff', 
            padding: '2px 6px', 
            borderRadius: '4px',
            border: '1px solid rgba(0, 229, 255, 0.3)',
            fontFamily: 'Orbitron'
          }}>
            {t.phase} {thread.task_kind === 'HeaderDecl' ? '1' : thread.task_kind === 'SourceImpl' ? '2' : '3'}
          </span>
        )}
        {title}
      </h3>

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ fontSize: '12px', color: '#888' }}>
          {t.updated} {updated_at}
        </div>
        
        {rejection_count !== undefined && rejection_count > 0 && (
          <div style={{ 
            fontSize: '11px', 
            color: '#ff4444', 
            backgroundColor: 'rgba(255, 68, 68, 0.1)',
            padding: '2px 8px',
            borderRadius: '4px',
            border: rejection_count >= 3 ? '2px solid #ff0000' : '1px solid rgba(255, 68, 68, 0.3)',
            animation: rejection_count >= 3 ? 'pulse-red 1s infinite' : 'none',
            fontWeight: rejection_count >= 3 ? 'bold' : 'normal'
          }}>
            {rejection_count >= 3 ? '🚨 ALERT!!! ' : ''}{rejection_count} {t.rejections}
          </div>
        )}
      </div>
      
      {status === 'Working' && (
        <div style={{
          position: 'absolute',
          bottom: 0,
          left: 0,
          height: '2px',
          width: '100%',
          background: 'linear-gradient(90deg, transparent, #ffd700, transparent)',
          animation: 'scan 2s linear infinite'
        }} />
      )}
    </div>
  );
};

export default ThreadCard;
