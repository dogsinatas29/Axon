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

export type ThreadStatus = 'Draft' | 'JuniorProposal' | 'SeniorReview' | 'PatchReady' | 'BossApproval' | 'Completed' | 'Paused' | 'Working' | 'Approved';

export interface Thread {
  id: string;
  title: string;
  status: ThreadStatus;
  author: string;
  milestone_id?: string;
  created_at: string;
  updated_at: string;
}

export type PostType = 'Proposal' | 'Review' | 'Patch' | 'Nogari' | 'System' | 'Instruction';

export interface Post {
  id: string;
  thread_id: string;
  author_id: string; // Agent ID or "BOSS"
  content: string;
  post_type: PostType;
  created_at: string;
}

export interface AgentPersona {
  name: string;
  character_core: string;
  prefixes: string[];
  suffixes: string[];
  description: string;
}

export type AgentRole = 'Architect' | 'Senior' | 'Junior';

export interface Agent {
  id: string;
  role: AgentRole;
  persona: AgentPersona;
  status: 'Idle' | 'Working' | 'Thinking';
  parent_id?: string;
  dtr: number;
  model?: string;
}

export interface Event {
  id: string;
  thread_id?: string;
  agent_id?: string;
  event_type: 'ThreadCreated' | 'ThreadStatusChanged' | 'PostAdded' | 'PatchCreated' | 'AgentAction' | 'SystemLog' | 'Signal';
  content: string;
  timestamp: string;
}
