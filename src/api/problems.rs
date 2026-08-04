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

pub mod get {
    use std::sync::Arc;

    use axum::Json;
    use axum::body::Body;
    use axum::extract::{Path, State};
    use axum::response::Response;
    use axum_anyhow::{ApiResult, OptionExt};

    use crate::api::problems::ProblemStatus;
    use crate::app;

    pub async fn problems(
        State(st): State<Arc<app::State>>,
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

        let q = sqlx::query!(
            r#"
            SELECT subtasks.subtask, subtasks.score
            FROM subtasks
            INNER JOIN subtask_testcases ON subtasks.subtask = subtask_testcases.FK_subtasks_subtask
            INNER JOIN submission_testcases ON submission_testcases.FK_submissions_id = subtask_testcases.FK_subtasks_subtask
            INNER JOIN submissions ON submissions.FK_problems_id = subtasks.FK_problems_id
            WHERE subtasks.FK_problems_id = ?
            "#,
            problem_id
        );

        let num_solves = sqlx::query!(
            r#"
            SELECT DISTINCT
                submissions.FK_users_id
            FROM
                problems
            INNER JOIN
                submissions
                ON submissions.FK_problems_id = problems.id
            INNER JOIN
                subtask_testcases
                ON subtask_testcases.FK_subtasks_problems_id = submissions.FK_problems_id
            INNER JOIN
                submission_testcases
                ON submission_testcases.FK_submissions_id = submissions.id AND submission_testcases.testcase = subtask_testcases.testcase
            WHERE problems.id = ?
            GROUP BY submissions.id
            HAVING SUM(submission_testcases.status) = 0
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

        todo!()
    }
}
