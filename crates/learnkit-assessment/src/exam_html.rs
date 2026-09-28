use crate::item::{Assessment, AssessmentItem};

/// Renders a self-contained HTML exam: no external assets, no network, no
/// server — works by opening the file directly in a browser
/// (`docs/08-assessment-games-progress.md §6`). Scoring happens client-side;
/// finishing offers a "download results" button producing a JSON file in
/// the shape `attempt::import_results` expects (FR-023/FR-024).
pub fn render(assessment: &Assessment, items: &[AssessmentItem]) -> String {
    let items_json = serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string());
    let assessment_id = html_escape(&assessment.id);
    let title = html_escape(&assessment.title);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{title}</title>
<style>
  body {{ font-family: sans-serif; max-width: 640px; margin: 2rem auto; padding: 0 1rem; }}
  .question {{ margin-bottom: 1.5rem; padding: 1rem; border: 1px solid #ccc; border-radius: 8px; }}
  .options label {{ display: block; margin: 0.25rem 0; }}
  #score {{ font-size: 1.25rem; font-weight: bold; }}
  button {{ padding: 0.5rem 1rem; font-size: 1rem; }}
</style>
</head>
<body>
<h1>{title}</h1>
<form id="exam-form"></form>
<button id="finish-btn" type="button">Finish exam</button>
<div id="result" style="display:none;">
  <p id="score"></p>
  <a id="download-link" download="results.json">Download results</a>
</div>
<script>
const ASSESSMENT_ID = "{assessment_id}";
const ITEMS = {items_json};

function renderForm() {{
  const form = document.getElementById("exam-form");
  ITEMS.forEach((item, idx) => {{
    const div = document.createElement("div");
    div.className = "question";
    const prompt = document.createElement("p");
    prompt.textContent = (idx + 1) + ". " + item.prompt + " [" + item.skill + "]";
    div.appendChild(prompt);
    const optionsDiv = document.createElement("div");
    optionsDiv.className = "options";
    item.options.forEach((opt) => {{
      const label = document.createElement("label");
      const input = document.createElement("input");
      input.type = "radio";
      input.name = item.id;
      input.value = opt.id;
      label.appendChild(input);
      label.appendChild(document.createTextNode(" " + opt.text));
      optionsDiv.appendChild(label);
    }});
    div.appendChild(optionsDiv);
    form.appendChild(div);
  }});
}}

function finishExam() {{
  const attempts = ITEMS.map((item) => {{
    const selected = document.querySelector('input[name="' + item.id + '"]:checked');
    const selectedId = selected ? selected.value : null;
    const correct = selectedId !== null && item.correct_option_ids.includes(selectedId);
    return {{
      item_id: item.id,
      skill: item.skill,
      selected_option_ids: selectedId ? [selectedId] : [],
      correct: correct,
      score: correct ? 1.0 : 0.0
    }};
  }});
  const totalScore = attempts.reduce((sum, a) => sum + a.score, 0);
  const pct = ITEMS.length > 0 ? Math.round((totalScore / ITEMS.length) * 100) : 0;
  document.getElementById("score").textContent = "Score: " + totalScore + "/" + ITEMS.length + " (" + pct + "%)";
  document.getElementById("result").style.display = "block";

  const payload = {{ assessment_id: ASSESSMENT_ID, attempts: attempts }};
  const blob = new Blob([JSON.stringify(payload, null, 2)], {{ type: "application/json" }});
  const url = URL.createObjectURL(blob);
  const link = document.getElementById("download-link");
  link.href = url;
}}

document.getElementById("finish-btn").addEventListener("click", finishExam);
renderForm();
</script>
</body>
</html>
"##
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::{Option_, Skill};

    #[test]
    fn renders_self_contained_html_with_no_external_references() {
        let assessment = Assessment {
            id: "exam-001".to_string(),
            title: "Unit 5 exam".to_string(),
            item_ids: vec!["q-1".to_string()],
        };
        let items = vec![AssessmentItem {
            id: "q-1".to_string(),
            learning_item_ids: vec!["li-1".to_string()],
            item_type: "multiple_choice".to_string(),
            prompt: "What does \"whiteboard\" mean?".to_string(),
            prompt_audio_asset_id: None,
            options: vec![
                Option_ {
                    id: "opt-correct".to_string(),
                    text: "pizarra".to_string(),
                },
                Option_ {
                    id: "opt-distractor".to_string(),
                    text: "silla".to_string(),
                },
            ],
            correct_option_ids: vec!["opt-correct".to_string()],
            skill: Skill::Recognition,
        }];

        let html = render(&assessment, &items);

        assert!(html.contains("<html"));
        assert!(!html.contains("http://"));
        assert!(!html.contains("https://"));
        assert!(html.contains("whiteboard"));
        assert!(html.contains("results.json"));
    }
}
