        // E. Entrypoint Integrity Gate (v0.0.28)
        if comp.is_entrypoint && ir_data.components.len() > 1 {
            let mut calls_others = false;
            for (other_path, other_comp) in &ir_data.components {
                if other_path == file_path { continue; }
                for func in other_comp.functions.values() {
                    if content.contains(&func.name) {
                        calls_others = true;
                        break;
                    }
                }
                if calls_others { break; }
            }

            if !calls_others {
                return Err(anyhow::anyhow!(
                    "ENTRYPOINT COLLAPSE: 'main.c' is a trivial placeholder and does NOT integrate with other modules. Global integration logic is missing.",
                ));
            }
        }
