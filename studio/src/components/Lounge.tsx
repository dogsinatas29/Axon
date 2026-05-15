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
import { MessageSquare, Terminal } from 'lucide-react';
import type { Event } from '../types';

interface LoungeProps {
  events: Event[];
  t: any;
}

const Lounge: React.FC<LoungeProps> = ({ events, t }) => {
  return (
    <section className="panel">
      <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', alignItems: 'center' }}>
          <MessageSquare size={14} style={{ marginRight: '0.5rem' }} />
          {t.nogariLounge}
        </div>
        <Terminal size={14} color="var(--text-dim)" />
      </div>
      <div style={{ flex: 1, padding: '1rem', overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '0.75rem', fontSize: '0.8rem' }}>
        {events.filter(ev => ev.event_type === 'MessagePosted').map((ev) => (
          <div key={ev.id} style={{ borderLeft: '2px solid var(--accent-primary)', paddingLeft: '1rem', marginBottom: '1rem', background: 'rgba(255,255,255,0.02)', padding: '1rem', borderRadius: '4px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
               <span style={{ color: 'var(--accent-primary)', fontSize: '0.75rem', fontWeight: 'bold' }}>{ev.source || 'AGENT'}</span>
               <span style={{ color: 'var(--text-dim)', fontSize: '0.65rem' }}>{new Date(ev.timestamp).toLocaleTimeString()}</span>
            </div>
            <div style={{ color: 'var(--text-main)', lineHeight: '1.6', fontSize: '0.9rem', fontStyle: 'italic' }}>
              {ev.content.replace('💬', '').trim()}
            </div>
          </div>
        ))}
        {events.filter(ev => ev.event_type === 'MessagePosted').length === 0 && (
            <div style={{ textAlign: 'center', color: 'var(--text-dim)', fontSize: '0.8rem', marginTop: '2rem', opacity: 0.5 }}>
                <MessageSquare size={32} style={{ marginBottom: '1rem', opacity: 0.2 }} />
                <p>{t.agentsBusy}<br/>{t.nogariSoon}</p>
            </div>
        )}
      </div>
    </section>
  );
};

export default Lounge;
