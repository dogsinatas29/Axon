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
}

const Lounge: React.FC<LoungeProps> = ({ events }) => {
  return (
    <section className="panel">
      <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', alignItems: 'center' }}>
          <MessageSquare size={14} style={{ marginRight: '0.5rem' }} />
          Nogari Lounge
        </div>
        <Terminal size={14} color="var(--text-dim)" />
      </div>
      <div style={{ flex: 1, padding: '1rem', overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '0.75rem', fontSize: '0.8rem' }}>
        {events.map((ev) => (
          <div key={ev.id} style={{ borderLeft: '2px solid rgba(255,255,255,0.05)', paddingLeft: '0.5rem' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.2rem' }}>
               <span style={{ color: 'var(--accent-primary)', fontSize: '0.7rem', fontWeight: 'bold' }}>{ev.event_type}</span>
               <span style={{ color: 'var(--text-dim)', fontSize: '0.6rem' }}>{new Date(ev.timestamp).toLocaleTimeString()}</span>
            </div>
            <span style={{ color: 'var(--text-main)', lineHeight: '1.4' }}>{ev.content}</span>
          </div>
        ))}
        {events.length === 0 && (
            <div style={{ textAlign: 'center', color: 'var(--text-dim)', fontSize: '0.7rem', marginTop: '1rem' }}>
                Waiting for agent signals...
            </div>
        )}
      </div>
    </section>
  );
};

export default Lounge;
