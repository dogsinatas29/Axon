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
import { X, Check, Shield, Clock, RotateCcw, ThumbsDown } from 'lucide-react';
import type { Thread, Post } from '../types';

interface ThreadDetailProps {
  thread: Thread;
  onClose: () => void;
  // onApprove: (id: string) => void;
  t: any;
  onRefresh?: () => void;
}

const ThreadDetail: React.FC<ThreadDetailProps> = ({ thread, onClose, t, onRefresh }) => {
  const [posts, setPosts] = useState<Post[]>([]);
  const [loading, setLoading] = useState(true);
  const [retryFeedback, setRetryFeedback] = useState('');

  useEffect(() => {
    const fetchPosts = () => {
      fetch(`/api/threads/${thread.id}/posts`)
        .then(res => res.json())
        .then(data => {
          setPosts(data);
          setLoading(false);
        })
        .catch(err => console.error('Failed to fetch posts', err));
    };
    fetchPosts();
    const interval = setInterval(fetchPosts, 2000);
    return () => clearInterval(interval);
  }, [thread.id]);

  const formatContent = (content: string) => {
    // 1. Unescape logic (Handle common AI output escapes)
    let processed = content
      .replace(/\\n/g, '\n')
      .replace(/\\t/g, '\t');

    // 2. Detect Code Block (Markdown style)
    if (processed.includes('```')) {
      const parts = processed.split('```');
      return (
        <div className="thought-content">
          {parts.map((part, idx) => {
            if (idx % 2 === 1) {
              // This is code
              const lines = part.split('\n');
              const lang = lines[0].trim();
              const code = lines.slice(1).join('\n');
              return (
                <div key={idx} className="code-block-container">
                  <div className="code-header">{lang || 'code'}</div>
                  <pre className="code-content"><code>{code}</code></pre>
                </div>
              );
            } else {
              // This is text
              return part.split('\n').map((line, i) => (
                <p key={`${idx}-${i}`} style={{ marginBottom: line ? '0.5rem' : '1rem' }}>
                  {line}
                </p>
              ));
            }
          })}
        </div>
      );
    }

    // 3. Detect Diff/Patch
    const isDiff = processed.includes('@@') || processed.includes('diff --git');
    
    if (isDiff) {
      const lines = processed.split('\n');
      return (
        <div className="advanced-diff-viewer">
          {lines.map((line, idx) => {
            let lineClass = 'diff-line';
            if (line.startsWith('+')) lineClass += ' added';
            else if (line.startsWith('-')) lineClass += ' removed';
            else if (line.startsWith('@@')) lineClass += ' meta';
            
            return (
              <div key={idx} className={lineClass}>
                <span className="line-num">{idx + 1}</span>
                <span className="line-content">{line}</span>
              </div>
            );
          })}
        </div>
      );
    }

    // 4. Regular text with line breaks
    return (
      <div className="thought-content">
        {processed.split('\n').map((line, i) => (
          <p key={i} style={{ marginBottom: line ? '0.5rem' : '1rem' }}>
            {line}
          </p>
        ))}
      </div>
    );
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

      <div className="posts-container" style={{ flex: 1, overflowY: 'auto', minHeight: 0 }}>
        {loading ? (
          <div className="loading">{t.analyzingHistory}</div>
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

      <div className="detail-footer" style={{ flexDirection: 'column', gap: '0.5rem' }}>
        {thread.status === 'BossApproval' && (
          <>
            <div style={{ display: 'flex', gap: '0.5rem', width: '100%' }}>
              <button
                className="btn-approve"
                style={{ flex: 1 }}
                onClick={async () => {
                  try {
                    await fetch(`/api/threads/${thread.id}/approve`, { method: 'POST' });
                    onRefresh?.();
                    onClose();
                  } catch (err) {
                    console.error('Approve failed', err);
                  }
                }}
              >
                <Check size={18} /> {t.approveLock || 'Approve'}
              </button>
              <button
                style={{ flex: 1, background: 'rgba(255,68,68,0.1)', color: '#ff4444', border: '1px solid rgba(255,68,68,0.3)', padding: '0.6rem 1rem', borderRadius: '8px', fontWeight: 'bold', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.4rem' }}
                onClick={async () => {
                  try {
                    await fetch(`/api/threads/${thread.id}/reject`, { method: 'POST' });
                    onRefresh?.();
                    onClose();
                  } catch (err) {
                    console.error('Reject failed', err);
                  }
                }}
              >
                <ThumbsDown size={18} /> {t.reject || 'Reject'}
              </button>
            </div>
            <div style={{ display: 'flex', gap: '0.5rem', width: '100%' }}>
              <input
                value={retryFeedback}
                onChange={(e) => setRetryFeedback(e.target.value)}
                placeholder="Retry feedback (optional)..."
                style={{ flex: 1, background: '#1a1a1a', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '6px', padding: '0.5rem', color: 'white', fontSize: '0.8rem' }}
              />
              <button
                style={{ background: 'rgba(0, 242, 255, 0.1)', color: 'var(--accent-primary)', border: '1px solid var(--accent-primary)', padding: '0.5rem 1rem', borderRadius: '6px', fontWeight: 'bold', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '0.4rem', whiteSpace: 'nowrap' }}
                onClick={async () => {
                  try {
                    await fetch(`/api/threads/${thread.id}/retry`, {
                      method: 'POST',
                      headers: { 'Content-Type': 'application/json' },
                      body: JSON.stringify({ feedback: retryFeedback || null }),
                    });
                    onRefresh?.();
                    onClose();
                  } catch (err) {
                    console.error('Retry failed', err);
                  }
                }}
              >
                <RotateCcw size={18} /> Retry
              </button>
            </div>
          </>
        )}
        <button className="btn-secondary" onClick={onClose}>{t.standBy}</button>
      </div>
    </motion.div>
  );
};

export default ThreadDetail;
