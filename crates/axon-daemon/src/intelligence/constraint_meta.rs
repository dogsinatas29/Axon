/// Metadata attached to a constraint to track its reliability and priority.
/// Used for tie-breaking during merge and filtering during pruning.
#[derive(Debug, Clone)]
pub struct ConstraintMeta {
    pub priority: f32,   // 초기값 1.0, range: 0.1 ~ 10.0
    pub confidence: f32, // 0.0 ~ 1.0, hit_rate 기반
    pub hits: u32,       // 검증에서 유효하게 사용된 횟수
    pub failures: u32,   // 위반이 검출된 횟수
    pub last_used: u64,  // Unix epoch (seconds)
}

impl Default for ConstraintMeta {
    fn default() -> Self {
        Self {
            priority: 1.0,
            confidence: 0.5,
            hits: 0,
            failures: 0,
            last_used: 0,
        }
    }
}

impl ConstraintMeta {
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates priority and confidence based on usage outcome.
    /// - `used`: constraint was evaluated this cycle
    /// - `violated`: a violation was caught (the constraint was useful)
    pub fn update(&mut self, used: bool, violated: bool, now: u64) {
        if used { self.hits += 1; }
        if violated { self.failures += 1; }

        // Useful detection → boost; unused / noise → decay
        let gain = if used && violated {
            0.1 // caught a real problem — valuable
        } else if used && !violated {
            0.02 // satisfied, still relevant
        } else {
            -0.05 // not triggered — may be stale
        };

        self.priority = (self.priority + gain).clamp(0.1, 10.0);

        // Confidence = success rate (hit without raising false alarm)
        let total = (self.hits + self.failures).max(1) as f32;
        self.confidence = self.hits as f32 / total;

        self.last_used = now;
    }

    /// A constraint is prunable if it's low-priority and hasn't been used recently.
    pub fn should_prune(&self, now: u64, ttl_secs: u64) -> bool {
        self.priority < 0.3 && (now.saturating_sub(self.last_used)) > ttl_secs
    }
}
