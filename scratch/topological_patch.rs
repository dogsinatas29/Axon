                                        // v0.0.28: TOPOLOGICAL FILTER (Exclude Entrypoints from ImplGen)
                                        let valid_tasks: Vec<axon_core::Task> = valid_tasks.into_iter()
                                            .filter(|t| {
                                                if let Some(ref f) = t.target_file {
                                                    if let Some(comp) = ir.get_component(f) {
                                                        return !comp.is_entrypoint;
                                                    }
                                                }
                                                true
                                            })
                                            .collect();
