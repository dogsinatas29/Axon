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
import { Terminal, Send, ShieldCheck } from 'lucide-react';

const BossBoard: React.FC = () => {
  const [spec, setSpec] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!spec.trim()) return;

    try {
      const response = await fetch('http://localhost:8080/api/specs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: spec }),
      });

      if (response.ok) {
        alert('새로운 명세가 공장에 하달되었습니다. 에이전트들이 분석을 시작합니다.');
        setSpec('');
      } else {
        alert('명세 하달 중 오류가 발생했습니다.');
      }
    } catch (error) {
      console.error('Error submitting spec:', error);
      alert('서버 연결에 실패했습니다.');
    }
  };

  return (
    <section className="panel" style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
      <div className="panel-header">
        <ShieldCheck size={16} style={{ marginRight: '0.5rem' }} />
        사장 게시판 (Boss Board) - Spec Declaration
      </div>
      
      <div style={{ padding: '1.5rem', flex: 1, display: 'flex', flexDirection: 'column', gap: '2rem' }}>
        <div className="card" style={{ border: '1px border var(--accent-primary)' }}>
            <h3 style={{ fontSize: '1rem', marginBottom: '1rem', color: 'var(--accent-primary)' }}>🚀 New Specification Declaration</h3>
            <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                <textarea 
                    value={spec}
                    onChange={(e) => setSpec(e.target.value)}
                    placeholder="공장에 하달할 새로운 기능 명세나 마일스톤을 입력하세요... (예: 'v0.0.3: 데이터베이스 마이그레이션 로직 구현')"
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
                    <Send size={16} /> 명령 하달 (Submit Spec)
                </button>
            </form>
        </div>

        <div className="card">
            <h3 style={{ fontSize: '0.9rem', marginBottom: '1rem', opacity: 0.8 }}>
                <Terminal size={14} style={{ marginRight: '0.5rem' }} />
                Bug Repository & Interrupts
            </h3>
            <div style={{ color: 'var(--text-dim)', fontSize: '0.8rem', textAlign: 'center', padding: '2rem' }}>
                No critical bug reports or interrupts pending.
            </div>
        </div>
      </div>
    </section>
  );
};

export default BossBoard;
