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
}

const ThreadCard: React.FC<ThreadCardProps> = ({ thread, onClick }) => {
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
        border: `1px solid ${config.color}`,
        boxShadow: config.glow,
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
        fontWeight: '600'
      }}>
        {title}
      </h3>

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ fontSize: '12px', color: '#888' }}>
          Updated {updated_at}
        </div>
        
        {rejection_count !== undefined && rejection_count > 0 && (
          <div style={{ 
            fontSize: '11px', 
            color: '#ff4444', 
            backgroundColor: 'rgba(255, 68, 68, 0.1)',
            padding: '2px 8px',
            borderRadius: '4px',
            border: '1px solid rgba(255, 68, 68, 0.3)'
          }}>
            {rejection_count} REJECTIONS
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
