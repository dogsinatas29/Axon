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

import React from 'react';
import { motion } from 'framer-motion';
import { MessageSquare, Clock, CheckCircle, AlertTriangle, Shield } from 'lucide-react';
import type { Thread, ThreadStatus } from '../types';

interface ThreadCardProps {
  thread: Thread;
  onClick?: (id: string) => void;
}

const ThreadCard: React.FC<ThreadCardProps> = ({ thread, onClick }) => {
  const getStatusInfo = (status: ThreadStatus) => {
    switch (status) {
      case 'Draft': return { color: '#666', icon: <Clock size={12} />, label: 'Draft' };
      case 'JuniorProposal': return { color: 'var(--accent-primary)', icon: <MessageSquare size={12} />, label: 'Proposal' };
      case 'SeniorReview': return { color: 'var(--accent-secondary)', icon: <AlertTriangle size={12} />, label: 'Reviewing' };
      case 'PatchReady': return { color: '#00ff95', icon: <CheckCircle size={12} />, label: 'Patch Ready' };
      case 'BossApproval': return { color: '#ffcc00', icon: <Shield size={12} />, label: 'BOSS Final' };
      case 'Completed': return { color: '#aaa', icon: <CheckCircle size={12} />, label: 'Closed' };
      default: return { color: '#333', icon: null, label: status };
    }
  };

  const info = getStatusInfo(thread.status);

  return (
    <motion.div 
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      whileHover={{ scale: 1.02 }}
      className="card"
      onClick={() => onClick?.(thread.id)}
      style={{ cursor: 'pointer', borderColor: info.color + '44' }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
        <span style={{ fontSize: '0.6rem', color: 'var(--text-dim)' }}>{thread.id}</span>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.3rem', fontSize: '0.65rem', color: info.color }}>
          {info.icon}
          {info.label}
        </div>
      </div>
      <h3 style={{ fontSize: '0.9rem', marginBottom: '0.5rem', fontFamily: 'Orbitron' }}>{thread.title}</h3>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '1rem' }}>
        <span style={{ fontSize: '0.6rem', color: 'var(--text-dim)' }}>
          Updated {new Date(thread.updated_at).toLocaleTimeString()}
        </span>
      </div>
    </motion.div>
  );
};

export default ThreadCard;
