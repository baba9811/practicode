use super::*;

pub fn give_up(root: &Path, problem: &Problem, state: &mut AppState) -> Result<String> {
    let language = normalize_language(&state.settings.language);
    let answer = problem
        .answers
        .get(&language)
        .cloned()
        .unwrap_or_else(|| problem.answers.values().next().cloned().unwrap_or_default());
    mark_history(state, &problem.id, "gave_up");
    upsert_problem_index(root, problem, "gave_up")?;
    save_state(root, state)?;
    Ok(answer)
}

pub fn next_problem(
    root: &Path,
    bank: &[Problem],
    state: &mut AppState,
) -> Result<Option<Problem>> {
    let seen = state
        .history
        .iter()
        .map(|item| item.id.as_str())
        .collect::<Vec<_>>();
    let preferred = if state.settings.difficulty == "auto" {
        &state.suggested_next_difficulty
    } else {
        &state.settings.difficulty
    };
    let problem = bank
        .iter()
        .find(|item| !seen.contains(&item.id.as_str()) && &item.difficulty == preferred)
        .or_else(|| bank.iter().find(|item| !seen.contains(&item.id.as_str())));
    let Some(problem) = problem.cloned() else {
        return Ok(None);
    };
    state.current_problem = problem.id.clone();
    mark_history(state, &problem.id, "assigned");
    save_state(root, state)?;
    ensure_problem_files(root, &problem)?;
    upsert_problem_index(root, &problem, "assigned")?;
    Ok(Some(problem))
}

pub fn previous_problem(root: &Path, bank: &[Problem], state: &mut AppState) -> Result<Problem> {
    let known_ids = bank
        .iter()
        .map(|problem| problem.id.as_str())
        .collect::<Vec<_>>();
    let history = state
        .history
        .iter()
        .filter(|item| known_ids.contains(&item.id.as_str()))
        .map(|item| item.id.clone())
        .collect::<Vec<_>>();
    let Some(index) = history.iter().position(|id| id == &state.current_problem) else {
        return problem_by_id(bank, &state.current_problem)
            .cloned()
            .ok_or_else(|| anyhow!("current problem missing"));
    };
    if index == 0 {
        return problem_by_id(bank, &state.current_problem)
            .cloned()
            .ok_or_else(|| anyhow!("current problem missing"));
    }
    state.current_problem = history[index - 1].clone();
    save_state(root, state)?;
    problem_by_id(bank, &state.current_problem)
        .cloned()
        .ok_or_else(|| anyhow!("current problem missing"))
}

pub fn record_pass(root: &Path, problem: &Problem, state: &mut AppState) -> Result<()> {
    if !state.solved.contains(&problem.id) {
        state.solved.push(problem.id.clone());
    }
    mark_history(state, &problem.id, "solved");
    upsert_problem_index(root, problem, "solved")?;
    state.suggested_next_difficulty = if state.solved.len() >= 2 {
        "medium".to_string()
    } else {
        "easy".to_string()
    };
    save_state(root, state)
}

pub fn mark_history(state: &mut AppState, problem_id: &str, status: &str) {
    if let Some(item) = state.history.iter_mut().find(|item| item.id == problem_id) {
        item.status = status.to_string();
    } else {
        state.history.push(HistoryItem {
            id: problem_id.to_string(),
            status: status.to_string(),
        });
    }
}
