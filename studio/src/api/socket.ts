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

let socket: WebSocket | null = null;
let listeners: ((data: any) => void)[] = [];

export const initSocket = (url: string) => {
  if (!socket) {
    // Convert http/https to ws/wss
    const wsUrl = url.replace(/^http/, 'ws') + '/ws';
    socket = new WebSocket(wsUrl);
    
    socket.onopen = () => {
      console.log('Connected to AXON Daemon (Native WS)');
    };

    socket.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        listeners.forEach(l => l(data));
      } catch (e) {
        console.error('Failed to parse WS message', e);
      }
    };

    socket.onclose = () => {
      console.log('Disconnected from AXON Daemon');
      socket = null;
    };
  }
  return {
    onEvent: (callback: (data: any) => void) => {
      listeners.push(callback);
    },
    disconnect: () => {
      socket?.close();
    }
  };
};

export const getSocket = () => socket;
