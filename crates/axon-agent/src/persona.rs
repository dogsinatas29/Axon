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

use rand::seq::SliceRandom;

#[allow(dead_code)]
pub struct AffixSystem {}

impl AffixSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_random(role: axon_core::AgentRole) -> axon_core::AgentPersona {
        let mut rng = rand::thread_rng();
        let prefixes = vec!["Cynical", "Enthusiastic", "Sharp", "Lazy", "Diligent"];
        let cores = vec!["Architect", "Hacker", "Code-Monstrosity", "Optimization-Slave"];
        let suffixes = vec!["Coffee-addict", "Perfectionist", "Sleep-deprived", "Gopher-fan"];

        let prefix = prefixes.choose(&mut rng).unwrap().to_string();
        let core = cores.choose(&mut rng).unwrap().to_string();
        let suffix = suffixes.choose(&mut rng).unwrap().to_string();

        let genders = vec!["Male", "Female", "Non-binary"];
        let gender = genders.choose(&mut rng).unwrap().to_string();

        let name = format!("{} {}-{}", prefix, core, suffix);
        let description = match role {
            axon_core::AgentRole::Architect => format!("The visionary lead, {} and {}. Gender: {}.", prefix, suffix, gender),
            axon_core::AgentRole::Senior => format!("A senior agent who is {} and {}. Gender: {}.", prefix, suffix, gender),
            axon_core::AgentRole::Junior => format!("A junior agent striving to be a {}. Gender: {}.", core, gender),
        };

        axon_core::AgentPersona {
            name,
            gender,
            character_core: core,
            prefixes: vec![prefix],
            suffixes: vec![suffix],
            description,
        }
    }
}
