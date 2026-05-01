use crate::ir::*;

#[derive(Debug, Clone)]
pub enum IRChange {
    ReplaceComponent {
        component: Component,
    },

    DeleteComponent {
        name: String,
    },

    AddOrUpdateFunction {
        component: String,
        function: Function,
    },

    DeleteFunction {
        component: String,
        function_name: String,
    },
}

pub fn apply_changes(ir: &mut ProjectIR, changes: Vec<IRChange>) {
    for change in changes {
        match change {
            IRChange::DeleteComponent { name } => {
                ir.components.remove(&name);
            }

            IRChange::ReplaceComponent { component } => {
                ir.components.insert(component.name.clone(), component);
            }

            IRChange::AddOrUpdateFunction { component, function } => {
                if let Some(comp) = ir.components.get_mut(&component) {
                    comp.functions.insert(function.name.clone(), function);
                }
            }

            IRChange::DeleteFunction { component, function_name } => {
                if let Some(comp) = ir.components.get_mut(&component) {
                    comp.functions.remove(&function_name);
                }
            }
        }
    }
}
