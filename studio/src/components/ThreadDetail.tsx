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
import { motion } from 'framer-motion';
import { X, Check, Shield, Clock } from 'lucide-react';
import type { Thread, Post } from '../types';

interface ThreadDetailProps {
  thread: Thread;
  onClose: () => void;
  onApprove: (id: string) => void;
}

const ThreadDetail: React.FC<ThreadDetailProps> = ({ thread, onClose, onApprove }) => {
  const [posts, setPosts] = useState<Post[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch(`http://localhost:8080/api/threads/${thread.id}/posts`)
      .then(res => res.json())
      .then(data => {
        setPosts(data);
        setLoading(false);
      })
      .catch(err => console.error('Failed to fetch posts', err));
  }, [thread.id]);

  const formatContent = (content: string) => {
    // 1. Unescape logic (Handle common AI output escapes)
    let processed = content
      .replace(/\\n/g, '\n')
      .replace(/\\t/g, '\t');

    // 2. Detect Diff/Patch or Code block
    const isDiff = processed.includes('@@') || processed.includes('diff --git');
    
    if (isDiff) {
      return (
        <div className="code-render-block diff">
          <pre><code>{processed}</code></pre>
        </div>
      );
    }

    // 3. Regular text with line breaks
    return processed.split('\n').map((line, i) => (
      <span key={i}>
        {line}
        <br />
      </span>
    ));
  };

  return (
    <motion.div 
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 20 }}
      className="thread-detail-overlay"
    >
      <div className="detail-header">
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <Shield size={20} color="var(--accent-primary)" />
          <h2 style={{ fontFamily: 'Orbitron', fontSize: '1.1rem' }}>{thread.title}</h2>
          <span className={`status-badge ${thread.status.toLowerCase()}`}>{thread.status}</span>
        </div>
        <button onClick={onClose} className="btn-icon">
          <X size={20} />
        </button>
      </div>

      <div className="posts-container">
        {loading ? (
          <div className="loading">Analyzing transmission history...</div>
        ) : (
          posts.map(post => (
            <div key={post.id} className={`post-card ${post.post_type.toLowerCase()}`}>
              <div className="post-meta">
                <span className="author">{post.author_id}</span>
                <span className="time"><Clock size={12} /> {new Date(post.created_at).toLocaleTimeString()}</span>
              </div>
              <div className="post-content">
                {formatContent(post.content)}
              </div>
            </div>
          ))
        )}
      </div>

      <div className="detail-footer">
        {thread.status === 'BossApproval' && (
          <button 
            className="btn-approve"
            onClick={() => onApprove(thread.id)}
          >
            <Check size={18} /> APPROVE & LOCK
          </button>
        )}
        <button className="btn-secondary" onClick={onClose}>STAND BY</button>
      </div>
    </motion.div>
  );
};

export default ThreadDetail;
