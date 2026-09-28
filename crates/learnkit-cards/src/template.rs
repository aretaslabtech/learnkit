#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaPolicy {
    Required,
    Optional,
    Disabled,
}

/// Per-side, per-media-type policy for a card template, per
/// `docs/07-anki-media.md §5`.
#[derive(Debug, Clone, Copy)]
pub struct TemplateDefinition {
    pub id: &'static str,
    pub front_image: MediaPolicy,
    pub front_audio: MediaPolicy,
    pub back_image: MediaPolicy,
    pub back_audio: MediaPolicy,
}

/// Language profile templates, per `docs/06-profiles.md §3`.
pub const TEMPLATES: &[TemplateDefinition] = &[
    TemplateDefinition {
        id: "image-to-production-v1",
        front_image: MediaPolicy::Required,
        front_audio: MediaPolicy::Disabled,
        back_image: MediaPolicy::Disabled,
        back_audio: MediaPolicy::Optional,
    },
    TemplateDefinition {
        id: "audio-to-text-v1",
        front_image: MediaPolicy::Disabled,
        front_audio: MediaPolicy::Required,
        back_image: MediaPolicy::Disabled,
        back_audio: MediaPolicy::Disabled,
    },
    TemplateDefinition {
        id: "sentence-listening-v1",
        front_image: MediaPolicy::Disabled,
        front_audio: MediaPolicy::Required,
        back_image: MediaPolicy::Disabled,
        back_audio: MediaPolicy::Optional,
    },
    TemplateDefinition {
        id: "expression-production-v1",
        front_image: MediaPolicy::Optional,
        front_audio: MediaPolicy::Disabled,
        back_image: MediaPolicy::Disabled,
        back_audio: MediaPolicy::Required,
    },
    TemplateDefinition {
        id: "word-to-meaning-v1",
        front_image: MediaPolicy::Disabled,
        front_audio: MediaPolicy::Disabled,
        back_image: MediaPolicy::Disabled,
        back_audio: MediaPolicy::Disabled,
    },
];

pub fn find(id: &str) -> Option<&'static TemplateDefinition> {
    TEMPLATES.iter().find(|t| t.id == id)
}

pub fn is_supported(id: &str) -> bool {
    find(id).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_template_id_is_findable() {
        for t in TEMPLATES {
            assert!(is_supported(t.id));
        }
    }

    #[test]
    fn unknown_template_is_not_supported() {
        assert!(!is_supported("does-not-exist-v1"));
    }
}
