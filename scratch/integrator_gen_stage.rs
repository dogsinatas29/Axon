                Stage::IntegratorGen => {
                    tracing::info!("🔗 [STAGE:IntegratorGen] Finalizing Entry Point Integration... (Context: {})", context_size);
                    if let Some(ref ir) = ir_opt {
                        let mut symbols = Vec::new();
                        for comp in ir.components.values() {
                            if comp.is_entrypoint { continue; }
                            for func in comp.functions.values() {
                                symbols.push(format!("- {}: {}", comp.file_path, func.signature));
                            }
                        }
                        let registry = symbols.join("\n");
                        
                        let mut guide = daemon.architecture_guide.read().unwrap().clone();
                        guide.push_str("\n\n### 📦 GLOBAL SYMBOL REGISTRY (AVAILABLE FOR INTEGRATION) ###\n");
                        guide.push_str(&registry);
                        
                        let res = architect_runtime.process_bootstrap_step2_with_context(&serde_json::to_string(ir).unwrap(), context_size, Some(daemon.event_bus.clone())).await;
                        match res {
                            Ok(post) => {
                                let content = extract_json_block(post.content.trim());
                                if let Ok(d_tasks) = serde_json::from_str::<Vec<axon_core::DecomposedTask>>(&content) {
                                    let integrator_tasks: Vec<_> = d_tasks.into_iter()
                                        .map(|dt| {
                                            let mut t = axon_core::Task::from_decomposed(dt, self.project_id.clone());
                                            if let Some(ref cid) = t.target_file {
                                                if let Some(comp) = ir.components.get(cid) {
                                                    t.target_file = Some(comp.file_path.clone());
                                                }
                                            }
                                            t
                                        })
                                        .filter(|t| {
                                            if let Some(ref f) = t.target_file {
                                                if let Some(comp) = ir.get_component(f) {
                                                    return comp.is_entrypoint;
                                                }
                                            }
                                            false
                                        })
                                        .collect();
                                    
                                    if !integrator_tasks.is_empty() {
                                        tracing::info!("📥 [STAGE:IntegratorGen] Enqueuing {} integration tasks...", integrator_tasks.len());
                                        for mut task in integrator_tasks {
                                            task.id = format!("int_{}", task.id);
                                            task.task_kind = Some(axon_core::TaskKind::SourceImpl);
                                            
                                            let _ = daemon.storage.save_thread(axon_core::Thread {
                                                id: task.id.clone(),
                                                project_id: task.project_id.clone(),
                                                title: task.title.clone(),
                                                status: axon_core::ThreadStatus::Working,
                                                author: "Architect".to_string(),
                                                milestone_id: None,
                                                created_at: chrono::Local::now(),
                                                updated_at: chrono::Local::now(),
                                            }).await;

                                            let _ = daemon.storage.save_post(axon_core::Post {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                thread_id: task.id.clone(),
                                                author_id: "Architect".to_string(),
                                                content: format!("### 🔗 Integration Instruction\n\n**Goal**: {}\n\n**Integrator**: You are responsible for connecting all modules. Use the Global Symbol Registry provided.", task.title),
                                                thought: None,
                                                full_code: None,
                                                post_type: axon_core::PostType::Instruction,
                                                metrics: None,
                                                created_at: chrono::Local::now(),
                                            }).await;

                                            let _ = daemon.storage.save_task(task.clone()).await;
                                            daemon.submit_task(task.clone());
                                        }
                                        
                                        tracing::info!("⏳ [STAGE:IntegratorGen] Waiting for entry point to materialize...");
                                        let mut int_wait = 0;
                                        while daemon.storage.count_active_tasks_by_project(&self.project_id).unwrap_or(0) > 0 {
                                            if int_wait > 60 { break; }
                                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                            int_wait += 1;
                                        }
                                    }
                                }
                                stage = StageRouter::next_stage(&stage);
                                attempts = 0;
                            },
                            Err(_) => {
                                stage = Stage::Skeleton; 
                            }
                        }
                    } else {
                        stage = Stage::Skeleton;
                    }
                },
