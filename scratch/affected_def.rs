            let affected: Vec<String> = batch.dependency_closure.iter()
                .filter(|id| id.starts_with("file:"))
                .map(|id| id.replace("file:", ""))
                .collect();
