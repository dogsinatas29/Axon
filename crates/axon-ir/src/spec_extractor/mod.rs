// No imports needed here currently

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecKind {
    AxonSpec,           // <!-- AXON:SPEC:COMPONENTS -->
    MarkdownSpec,       // # Architecture / ## Components
    GenericSpec,        // spec: {...}
    JsonStructured,     // { "components": [...] }
    Unknown,
}

impl SpecKind {
    pub fn confidence(&self) -> f32 {
        match self {
            SpecKind::AxonSpec => 1.0,
            SpecKind::JsonStructured => 0.95,
            SpecKind::MarkdownSpec => 0.8,
            SpecKind::GenericSpec => 0.6,
            SpecKind::Unknown => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExtractedSpec {
    pub kind: SpecKind,
    pub content: String,
    pub confidence: f32,
    pub line_offset: usize,
}

impl ExtractedSpec {
    pub fn is_valid(&self) -> bool {
        self.confidence >= 0.5 && !self.content.is_empty()
    }
}

pub struct SpecExtractor;

impl SpecExtractor {
    pub fn extract(text: &str) -> Option<ExtractedSpec> {
        let normalized = text.to_lowercase();

        // Priority 1: AXON:SPEC block (highest confidence)
        if let Some(spec) = Self::extract_axon_spec(text, &normalized) {
            return Some(spec);
        }

        // Priority 2: JSON structured IR
        if let Some(spec) = Self::extract_json_struct(text, &normalized) {
            return Some(spec);
        }

        // Priority 3: Markdown architecture sections
        if let Some(spec) = Self::extract_markdown_spec(text, &normalized) {
            return Some(spec);
        }

        // Priority 4: Generic spec patterns
        if let Some(spec) = Self::extract_generic_spec(text, &normalized) {
            return Some(spec);
        }

        None
    }

    fn extract_axon_spec(text: &str, normalized: &str) -> Option<ExtractedSpec> {
        let patterns = [
            "<axon_spec>",
            "<axon-spec>",
            "<!-- axonspec",    // lowercase scan
            "<!-- axo:n:spec",
            "```spec",
            "```axonspec",
        ];

        for pattern in patterns {
            if let Some(start) = normalized.find(pattern) {
                let line_offset = text[..start].matches('\n').count();

                // 1. Try new sentinel tags
                if pattern.starts_with("<axon") {
                    let tag_name = pattern.trim_matches(|c| c == '<' || c == '>');
                    let end_tag = format!("</{}>", tag_name);
                    if let Some(end_idx) = normalized[start..].find(&end_tag) {
                        let content = text[start + pattern.len()..start + end_idx].trim().to_string();
                        return Some(ExtractedSpec {
                            kind: SpecKind::AxonSpec,
                            content,
                            confidence: 1.0,
                            line_offset,
                        });
                    }
                }

                // 2. Try classic <!-- --> comments
                if pattern.starts_with("<!--") {
                    if let Some(end_idx) = text[start..].find("-->") {
                        let content = text[start..start + end_idx + 3].trim().to_string();
                        return Some(ExtractedSpec {
                            kind: SpecKind::AxonSpec,
                            content,
                            confidence: 1.0,
                            line_offset,
                        });
                    }
                }

                // 3. Try fenced blocks
                if pattern.starts_with("```") {
                    if let Some(end_idx) = text[start + 3..].find("```") {
                        let content = text[start..start + end_idx + 6].trim().to_string();
                        return Some(ExtractedSpec {
                            kind: SpecKind::AxonSpec,
                            content,
                            confidence: 1.0,
                            line_offset,
                        });
                    }
                }

                // Last resort: Take a chunk
                let content = text[start..].chars().take(2000).collect();
                return Some(ExtractedSpec {
                    kind: SpecKind::AxonSpec,
                    content,
                    confidence: 0.7,
                    line_offset,
                });
            }
        }

        None
    }

    fn extract_json_struct(text: &str, normalized: &str) -> Option<ExtractedSpec> {
        // Look for JSON-like structure with components key
        if normalized.contains("\"components\"") || normalized.contains("\"components\":") {
            // Find JSON start
            let start = normalized.find("{")?;
            let end = text.rfind("}").map(|i| i + 1).unwrap_or(text.len());

            // Basic JSON extraction
            let content = text[start..end].trim().to_string();

            // Validate it looks like IR
            if content.contains("\"components\"") || content.contains("\"path\"") {
                return Some(ExtractedSpec {
                    kind: SpecKind::JsonStructured,
                    content,
                    confidence: SpecKind::JsonStructured.confidence(),
                    line_offset: text[..start].matches('\n').count(),
                });
            }
        }

        None
    }

    fn extract_markdown_spec(text: &str, normalized: &str) -> Option<ExtractedSpec> {
        // Look for architecture/markdown sections - EXPANDED FALLBACKS
        let markers = [
            // English
            "# architecture",
            "## components",
            "## component",
            "# design",
            "## structure",
            "# 아키텍처",
            "## 구성요소",
            "## 컴포넌트",
            "## 데이터 흐름",
            "# 설계",
            "## file",
            "### file",
            "## modules",
            "## functions",
            // Korean variations
            "## 파일",
            "## 함수",
            "## 모듈",
            // Generic
            "# 프로젝트 개요",
            "## 개요",
            "# overview",
        ];

        for marker in markers {
            if let Some(start) = normalized.find(marker) {
                let line_offset = text[..start].matches('\n').count();

                // Extract until next major section (# heading) or end
                let remaining = &text[start..];
                let mut end_pos = remaining.len();

                // Find next major heading
                for (_i, line) in remaining.lines().enumerate().skip(1) {
                    if line.starts_with("# ") || line.starts_with("## ") {
                        end_pos = start + remaining[..remaining.find(line).unwrap_or(0)].len();
                        break;
                    }
                }

                let content: String = remaining.chars().take(end_pos.min(start + 3000)).collect();

                // Check if it has ANY component-like markers
                let has_content = content.contains("file")
                    || content.contains("path")
                    || content.contains("파일")
                    || content.contains("함수")
                    || content.contains("function")
                    || content.contains("module")
                    || content.contains("모듈")
                    || content.contains("component")
                    || content.contains("- ");

                if has_content {
                    tracing::debug!("[SPEC_EXTRACTOR] Markdown fallback triggered: marker='{}'", marker);
                    return Some(ExtractedSpec {
                        kind: SpecKind::MarkdownSpec,
                        content,
                        confidence: SpecKind::MarkdownSpec.confidence(),
                        line_offset,
                    });
                }
            }
        }

        None
    }

    fn extract_generic_spec(text: &str, normalized: &str) -> Option<ExtractedSpec> {
        // Look for generic spec patterns - EXPANDED
        let patterns = [
            "spec:",
            "specification:",
            "```spec",
            "```json",
            "spec block",
            "architecture:",
        ];

        for pattern in patterns {
            if let Some(start) = normalized.find(pattern) {
                let line_offset = text[..start].matches('\n').count();
                let content: String = text[start..].chars().take(2000).collect();

                return Some(ExtractedSpec {
                    kind: SpecKind::GenericSpec,
                    content,
                    confidence: SpecKind::GenericSpec.confidence(),
                    line_offset,
                });
            }
        }

        // FINAL FALLBACK: If text has ANY component-like structure, accept it
        if text.len() > 100 {
            let has_structure =
                text.contains("- ") ||
                text.contains("file:") ||
                text.contains("path:") ||
                text.contains("함수") ||
                text.contains("파일") ||
                (text.matches('\n').count() > 5 && text.contains(":"));

            if has_structure {
                tracing::warn!("[SPEC_EXTRACTOR] FINAL_FALLBACK: Accepting raw text as GenericSpec");
                return Some(ExtractedSpec {
                    kind: SpecKind::GenericSpec,
                    content: text.chars().take(5000).collect(),
                    confidence: 0.3, // Low confidence but acceptable
                    line_offset: 0,
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axon_spec_detection() {
        let input = "Some text\n<!-- AXON:SPEC:COMPONENTS\n{\"components\": []} -->";
        let result = SpecExtractor::extract(input);
        assert!(result.is_some());
        assert_eq!(result.unwrap().kind, SpecKind::AxonSpec);
    }

    #[test]
    fn test_markdown_detection() {
        let input = "# Architecture\n## Components\n- File: test.c";
        let result = SpecExtractor::extract(input);
        assert!(result.is_some());
    }

    #[test]
    fn test_case_insensitive() {
        let input = "# Architecture\n# AXON:SPEC\n{ components: [] }";
        let result = SpecExtractor::extract(input);
        assert!(result.is_some());
    }
}