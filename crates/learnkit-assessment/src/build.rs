use crate::item::{AssessmentItem, Option_, Skill};
use learnkit_profile::language::learning_item::LearningItem;

fn distractor<'a>(
    items: &'a [LearningItem],
    current_index: usize,
    field: impl Fn(&'a LearningItem) -> &'a str,
) -> Option<&'a str> {
    if items.len() < 2 {
        return None;
    }
    let other_index = (current_index + 1) % items.len();
    if other_index == current_index {
        return None;
    }
    Some(field(&items[other_index]))
}

/// Generates a question bank covering recognition/production/listening for
/// every vocabulary `LearningItem`, per FR-021/FR-022. Distractors are drawn
/// from other items in the same bank (deterministic, no external model);
/// with fewer than two items, the question is still generated with a single
/// option rather than failing.
pub fn build_items(items: &[LearningItem]) -> Vec<AssessmentItem> {
    let mut generated = Vec::new();

    for (index, item) in items.iter().enumerate() {
        generated.push(recognition_item(items, index, item));
        generated.push(production_item(items, index, item));
        generated.push(listening_item(items, index, item));
    }

    generated
}

fn options_for(correct_text: &str, distractor_text: Option<&str>) -> (Vec<Option_>, Vec<String>) {
    let mut options = vec![Option_ {
        id: "opt-correct".to_string(),
        text: correct_text.to_string(),
    }];
    if let Some(text) = distractor_text {
        options.push(Option_ {
            id: "opt-distractor".to_string(),
            text: text.to_string(),
        });
    }
    (options, vec!["opt-correct".to_string()])
}

fn recognition_item(items: &[LearningItem], index: usize, item: &LearningItem) -> AssessmentItem {
    let distractor = distractor(items, index, |i| i.summary.as_str());
    let (options, correct) = options_for(&item.summary, distractor);
    AssessmentItem {
        id: format!("q-recognition-{}", item.id),
        learning_item_ids: vec![item.id.clone()],
        item_type: "multiple_choice".to_string(),
        prompt: format!("What does \"{}\" mean?", item.title),
        prompt_audio_asset_id: None,
        options,
        correct_option_ids: correct,
        skill: Skill::Recognition,
    }
}

fn production_item(items: &[LearningItem], index: usize, item: &LearningItem) -> AssessmentItem {
    let distractor = distractor(items, index, |i| i.title.as_str());
    let (options, correct) = options_for(&item.title, distractor);
    AssessmentItem {
        id: format!("q-production-{}", item.id),
        learning_item_ids: vec![item.id.clone()],
        item_type: "multiple_choice".to_string(),
        prompt: format!("How do you say \"{}\" in English?", item.summary),
        prompt_audio_asset_id: None,
        options,
        correct_option_ids: correct,
        skill: Skill::Production,
    }
}

fn listening_item(items: &[LearningItem], index: usize, item: &LearningItem) -> AssessmentItem {
    let distractor = distractor(items, index, |i| i.title.as_str());
    let (options, correct) = options_for(&item.title, distractor);
    AssessmentItem {
        id: format!("q-listening-{}", item.id),
        learning_item_ids: vec![item.id.clone()],
        item_type: "multiple_choice".to_string(),
        prompt: "Listen and choose what you hear.".to_string(),
        // No audio asset wired yet for assessment prompts in this feature;
        // the text fallback keeps the item usable without one (FR-022:
        // audio "cuando aplique", not unconditionally required).
        prompt_audio_asset_id: None,
        options,
        correct_option_ids: correct,
        skill: Skill::Listening,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, title: &str, summary: &str) -> LearningItem {
        LearningItem {
            id: id.to_string(),
            kind: "vocabulary".to_string(),
            title: title.to_string(),
            summary: summary.to_string(),
            tags: vec![],
            mastery_dimensions: vec![],
            vocabulary_entry_id: Some(format!("vocab-{id}")),
            source_ref: None,
        }
    }

    #[test]
    fn generates_three_skills_per_item() {
        let items = vec![
            item("li-1", "get away with", "hacer algo malo sin castigo"),
            item("li-2", "whiteboard", "pizarra"),
        ];

        let questions = build_items(&items);

        assert_eq!(questions.len(), 6);
        let skills: std::collections::HashSet<_> =
            questions.iter().map(|q| q.skill.as_str()).collect();
        assert_eq!(skills.len(), 3);
    }

    #[test]
    fn single_item_still_produces_valid_questions() {
        let items = vec![item("li-1", "whiteboard", "pizarra")];
        let questions = build_items(&items);

        assert_eq!(questions.len(), 3);
        for q in &questions {
            assert!(!q.options.is_empty());
            assert!(!q.correct_option_ids.is_empty());
        }
    }
}
