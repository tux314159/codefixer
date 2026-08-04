use serde::{Deserialize, Serialize};
use url::Url;

type UnixTime = i64; // signed time since the epoch in milliseconds

#[derive(Deserialize, Serialize)]
pub enum SubmissionPermitted {
    Allowed,
    LoginRequired,
    SubmissionLimitReached,
    CooldownReached,
    ProblemClosed,
}

#[derive(Deserialize, Serialize)]
pub struct SubmissionPolicy {
    pub allowed: SubmissionPermitted,
    pub allowed_languages: Vec<String>,
    pub max_source_bytes: i64,
    pub cooldown_seconds: i64,
    pub remaining_submissions: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub enum ProblemEditorial {
    Available(Url),
    NotPublished,
    ContestActive,
    SolveRequired,
}

#[derive(Deserialize, Serialize)]
pub enum ProblemType {
    Batch,
    Interactive,
    Communication,
}

#[derive(Deserialize, Serialize)]
pub enum ProblemStatus {
    Unattempted,
    Attempted(i64), // score
}

#[derive(Deserialize, Serialize)]
pub struct Problem {
    pub id: i64,
    pub title: String,
    pub difficulty: Option<i64>,
    pub problem_type: ProblemType,
    pub source: String,
    pub authors: Vec<String>,
    pub solves: i64,
    pub status: ProblemStatus,
    pub created_at: UnixTime,
    pub revision: String,
}

#[derive(Deserialize, Serialize)]
pub struct ProblemAttachment {
    pub name: String,
    pub content_type: String,
    pub size: u64, // 4GiB max, might have to change
    pub url: Url,
    pub expires_at: UnixTime,
}

#[derive(Deserialize, Serialize)]
pub struct ProblemDetails {
    pub statement_html: String,
    pub tags: Vec<String>,
    pub time_limit: i64,
    pub memory_limit: u64,
    pub attachments: Vec<ProblemAttachment>,
    pub subtasks: Vec<i64>,
    pub submission_policy: SubmissionPolicy,
    pub editorial: ProblemEditorial,
}

pub const PROBLEMS_URI: &str = "/api/v1/problems/{id}";

pub mod get {
    use std::sync::Arc;

    use axum::Json;
    use axum::extract::{Path, Extension};
    use axum_anyhow::{ApiResult, OptionExt};

    use crate::api::problems::ProblemStatus;
    use crate::app;

    pub async fn problems(
        Extension(st): Extension<Arc<app::State>>,
        Path(problem_id): Path<i64>,
    ) -> ApiResult<Json<super::Problem>> {
        struct ProblemRow {
            id: i64,
            title: String,
            source: String,
            tl: i64,
            ml: i64,
            runtype: i64,
            subtask: i64,
        }
        let problem = sqlx::query_as!(
            ProblemRow,
            r#"
            SELECT problems.id, problems.title, problems.source, problems.tl, problems.ml, problems.runtype, subtasks.subtask
            FROM problems
            INNER JOIN subtasks ON problems.id = subtasks.FK_problems_id
            WHERE id = ?
            "#,
            problem_id
        )
        .fetch_optional(&st.db_pool)
        .await?
        .context_not_found(format!("No such problem: {problem_id}"))?;

        let num_solves = sqlx::query!(
            r#"
            SELECT DISTINCT
                submissions.FK_users_id
            FROM
                problems
            INNER JOIN
                submissions
                ON submissions.FK_problems_id = problems.id
            WHERE problems.id = ? AND submissions.score = 100
            "#,
            problem_id
        ).fetch_all(&st.db_pool).await.iter().len();

        let r = super::Problem {
            id: problem_id,
            title: problem.title,
            difficulty: None,
            problem_type: super::ProblemType::Batch, // TODO
            source: problem.source,
            authors: vec![],
            solves: num_solves as i64,
            status: ProblemStatus::Unattempted,
            created_at: 0,
            revision: String::new(),
        };

        Ok(Json(r))
    }
}
