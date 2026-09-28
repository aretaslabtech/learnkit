use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Skill {
    Recognition,
    Production,
    Listening,
}

impl Skill {
    pub fn as_str(&self) -> &'static str {
        match self {
            Skill::Recognition => "recognition",
            Skill::Production => "production",
            Skill::Listening => "listening",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Option_ {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentItem {
    pub id: String,
    pub learning_item_ids: Vec<String>,
    #[serde(rename = "type")]
    pub item_type: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_audio_asset_id: Option<String>,
    pub options: Vec<Option_>,
    pub correct_option_ids: Vec<String>,
    pub skill: Skill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assessment {
    pub id: String,
    pub title: String,
    pub item_ids: Vec<String>,
}

fn assessments_dir(session_root: &Path) -> PathBuf {
    session_root.join("assessments")
}

fn items_dir(session_root: &Path) -> PathBuf {
    assessments_dir(session_root).join("items")
}

pub fn save_item(session_root: &Path, item: &AssessmentItem) -> std::io::Result<()> {
    let yaml = serde_yaml::to_string(item)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(
        &items_dir(session_root).join(format!("{}.yaml", item.id)),
        yaml.as_bytes(),
    )
}

pub fn load_all_items(session_root: &Path) -> std::io::Result<Vec<AssessmentItem>> {
    let dir = items_dir(session_root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let raw = fs::read_to_string(entry.path())?;
        let item: AssessmentItem = serde_yaml::from_str(&raw)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        items.push(item);
    }
    items.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(items)
}

pub fn save_assessment(session_root: &Path, assessment: &Assessment) -> std::io::Result<()> {
    let yaml = serde_yaml::to_string(assessment)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    learnkit_core::atomic::write_atomic(
        &assessments_dir(session_root).join(format!("{}.yaml", assessment.id)),
        yaml.as_bytes(),
    )
}

pub fn load_assessment(session_root: &Path, id: &str) -> std::io::Result<Option<Assessment>> {
    let path = assessments_dir(session_root).join(format!("{id}.yaml"));
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    let assessment: Assessment = serde_yaml::from_str(&raw)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    Ok(Some(assessment))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_item(id: &str) -> AssessmentItem {
        AssessmentItem {
            id: id.to_string(),
            learning_item_ids: vec!["li-1".to_string()],
            item_type: "multiple_choice".to_string(),
            prompt: "What does 'get away with' mean?".to_string(),
            prompt_audio_asset_id: None,
            options: vec![
                Option_ {
                    id: "opt-a".to_string(),
                    text: "hacer algo malo sin castigo".to_string(),
                },
                Option_ {
                    id: "opt-b".to_string(),
                    text: "irse de viaje".to_string(),
                },
            ],
            correct_option_ids: vec!["opt-a".to_string()],
            skill: Skill::Recognition,
        }
    }

    #[test]
    fn saves_and_loads_items() {
        let dir = tempfile::tempdir().unwrap();
        save_item(dir.path(), &sample_item("q-001")).unwrap();

        let items = load_all_items(dir.path()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "q-001");
    }

    #[test]
    fn saves_and_loads_an_assessment() {
        let dir = tempfile::tempdir().unwrap();
        let assessment = Assessment {
            id: "exam-001".to_string(),
            title: "Unit 5 exam".to_string(),
            item_ids: vec!["q-001".to_string()],
        };
        save_assessment(dir.path(), &assessment).unwrap();

        let loaded = load_assessment(dir.path(), "exam-001").unwrap().unwrap();
        assert_eq!(loaded.title, "Unit 5 exam");
    }
}
