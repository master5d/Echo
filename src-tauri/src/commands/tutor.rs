#[tauri::command]
#[specta::specta]
pub fn tutor_score(reference: String, spoken: String) -> crate::tutor::ScoreReport {
    crate::tutor::score_pronunciation(&reference, &spoken)
}
