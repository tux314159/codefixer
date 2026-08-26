use serde::{Deserialize, Serialize};
use url::Url;

type UnixTime = i64; // signed time since the epoch in milliseconds

#[derive(Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub enum SubmissionPermitted {
    Allowed,
    LoginRequired,
    SubmissionLimitReached,
    CooldownReached,
    ProblemClosed,
}

#[derive(Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct SubmissionPolicy {
    pub allowed: SubmissionPermitted,
    pub allowed_languages: Vec<String>,
    pub max_source_bytes: i64,
    pub cooldown_seconds: i64,
    pub remaining_submissions: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub enum ProblemEditorial {
    #[schema(value_type = String)]
    Available(Url),
    NotPublished,
    ContestActive,
    SolveRequired,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub enum ProblemType {
    Batch = 1,
    Interactive = 2,
    Communication = 3,
}

fn to_problem_type(t: i64) -> ProblemType {
    match t {
        1 => ProblemType::Batch,
        2 => ProblemType::Interactive,
        3 => ProblemType::Communication,
        _ => ProblemType::Batch,
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
// isomorphic to Option lol
pub enum ProblemStatus {
    Unattempted,
    Attempted(i64), // score
}

#[derive(Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ProblemSummary {
    pub id: i64,
    pub title: String,
    pub difficulty: Option<i64>,
    pub problem_type: ProblemType,
    pub source: String,
    pub authors: Vec<i64>,
    pub solves: i64,
    pub status: ProblemStatus,
    pub created_at: UnixTime,
}

#[derive(Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ProblemAttachment {
    pub name: String,
    pub content_type: String,
    pub size: i64,
    #[schema(value_type = String)]
    pub url: Url,
    pub expires_at: UnixTime,
}

#[derive(Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ProblemDetails {
    pub summary: ProblemSummary,
    pub statement_html: String,
    pub tags: Vec<String>,
    pub time_limit: i64,
    pub memory_limit: i64,
    pub attachments: Vec<ProblemAttachment>,
    pub subtasks: Vec<i64>,
    pub submission_policy: SubmissionPolicy,
    pub editorial: ProblemEditorial,
}

#[derive(utoipa::IntoParams, Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ProblemCollectionPage {
    items: Vec<ProblemSummary>,
}

pub const PROBLEMS_URI: &str = "/api/v1/problems";
pub const PROBLEMS_ID_URI: &str = "/api/v1/problems/{id}";

pub mod get {
    use std::sync::Arc;

    use axum::Json;
    use axum::extract::{Extension, Path};
    use axum_anyhow::{ApiResult, OptionExt};
    use axum_extra::extract::Query;
    use sea_query::JoinType;
    use sea_query::{Cond, Expr, ExprTrait, Func, NullOrdering, Order, SqliteQueryBuilder, all};
    use serde::Deserialize;
    use sqlx::AssertSqlSafe;
    use sqlx::prelude::FromRow;
    use tokio::fs;

    use crate::api::auth;
    use crate::app;

    use super::{
        ProblemCollectionPage, ProblemDetails, ProblemEditorial, ProblemStatus, ProblemSummary,
        ProblemType, SubmissionPermitted, SubmissionPolicy, to_problem_type,
    };

    #[derive(Copy, Clone, Debug, Deserialize)]
    pub enum ProblemSort {
        IdAsc,
        IdDesc,
        SolvesAsc,
        SolvesDesc,
        DifficultyAsc,
        DifficultyDesc,
        ScoreDesc,
        ScoreAsc,
        Newest,
        Oldest,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct ProblemsParams {
        q: Option<String>,
        limit: Option<i32>,
        page: Option<i32>,
        #[serde(default)]
        types: Vec<ProblemType>,
        #[serde(default)]
        tags: Vec<String>,
        min_difficulty: Option<i32>,
        max_difficulty: Option<i32>,
        rated: Option<bool>,
        solved: Option<bool>,
        sort: Option<ProblemSort>,
        selected: Option<i32>,
    }

    #[utoipa::path(
        get,
        path = super::PROBLEMS_URI,
        responses((status = OK, body = ProblemCollectionPage)),
    )]
    pub async fn problems(
        Extension(st): Extension<Arc<app::State>>,
        auth: auth::AuthSession,
        Query(params): Query<ProblemsParams>,
    ) -> ApiResult<Json<ProblemCollectionPage>> {
        let user = auth.user().await;

        #[derive(utoipa::ToSchema, FromRow)]
        struct QueryResult {
            id: i64,
            title: String,
            source: String,
            runtype: i64,
            created_at: i64,
            solves: i64,
            authors: Option<String>,
            max_score: Option<i64>,
        }
        let query = {
            // Query builder is not Send; we need a scope to drop the builder
            // before actually running the query.
            let mut query = sea_query::Query::select()
                .column(("problems", "id"))
                .column(("problems", "title"))
                .column(("problems", "source"))
                .column(("problems", "runtype"))
                .column(("problems", "created_at"))
                .column(("solves_tbl", "solves"))
                .expr_as(
                    Expr::cust("GROUP_CONCAT(DISTINCT problem_authors.FK_users_id)"),
                    "authors",
                )
                .column(("max_score_tbl", "max_score"))
                .from("problems")
                .left_join(
                    "problem_authors",
                    all![
                        Expr::col(("problems", "id")).equals(("problem_authors", "FK_problems_id")),
                    ],
                )
                .join_subquery(
                    JoinType::LeftJoin,
                    sea_query::Query::select()
                        .column("FK_problems_id")
                        .expr_as(Func::max(Expr::col("score")), "max_score")
                        .from("submissions")
                        .and_where(Expr::col("FK_users_id").eq(match &user {
                            None => -1,
                            Some(u) => u.id,
                        }))
                        .group_by_col("FK_problems_id")
                        .to_owned(),
                    "max_score_tbl",
                    all![Expr::col(("max_score_tbl", "FK_problems_id")).equals(("problems", "id"))],
                )
                .join_subquery(
                    JoinType::LeftJoin,
                    sea_query::Query::select()
                        .column("FK_problems_id")
                        .expr_as(Func::count_distinct(Expr::col("FK_users_id")), "solves")
                        .from("submissions")
                        .and_where(Expr::col("score").eq(100))
                        .group_by_col("FK_problems_id")
                        .to_owned(),
                    "solves_tbl",
                    all![Expr::col(("solves_tbl", "FK_problems_id")).equals(("problems", "id"))],
                )
                .group_by_col(("problems", "id"))
                .to_owned();

            if let Some(q) = params.q {
                query.and_where(
                    Func::upper(Expr::col(("problems", "title")))
                        .like(format!("%{}%", q.to_uppercase())),
                );
            }

            let limit = params.limit.unwrap_or(100);
            let limit = match limit {
                1..=9999 => limit,
                _ => 100,
            };
            query.limit(limit as u64);

            let page = params.page.unwrap_or(1);
            query.offset(((page - 1) * limit) as u64);

            if !params.types.is_empty() {
                let mut types_cond = Cond::any();
                for c in params
                    .types
                    .iter()
                    .map(|t| Expr::col(("problems", "runtype")).eq(*t as u64))
                {
                    types_cond = types_cond.add(c);
                }
                query.cond_where(types_cond);
            }

            if !params.tags.is_empty() {
                query.inner_join(
                    "problem_tags",
                    all![Expr::col(("problems", "id")).equals(("problem_tags", "FK_problems_id"))],
                );
            }

            if let Some(true) = params.solved {
                query.and_where(Expr::col("max_score").lt(100));
            }

            match params.sort.unwrap_or(ProblemSort::IdAsc) {
                ProblemSort::IdAsc => {
                    query.order_by_columns(vec![(("problems", "id"), Order::Asc)])
                }
                ProblemSort::IdDesc => {
                    query.order_by_columns(vec![(("problems", "id"), Order::Desc)])
                }
                ProblemSort::SolvesAsc => query.order_by_columns(vec![("solves", Order::Asc)]),
                ProblemSort::SolvesDesc => query.order_by_columns(vec![("solves", Order::Desc)]),
                ProblemSort::DifficultyAsc => {
                    // TODO
                    query.order_by_columns(vec![("solves", Order::Asc)])
                }
                ProblemSort::DifficultyDesc => {
                    // TODO
                    query.order_by_columns(vec![("solves", Order::Desc)])
                }
                ProblemSort::ScoreAsc => query.order_by_columns_with_nulls(vec![(
                    "max_score",
                    Order::Asc,
                    NullOrdering::Last,
                )]),
                ProblemSort::ScoreDesc => query.order_by_columns_with_nulls(vec![(
                    "max_score",
                    Order::Desc,
                    NullOrdering::Last,
                )]),
                ProblemSort::Newest => {
                    query.order_by_columns(vec![(("problems", "created_at"), Order::Asc)])
                }
                ProblemSort::Oldest => {
                    query.order_by_columns(vec![(("problems", "created_at"), Order::Desc)])
                }
            };
            query.order_by_columns(vec![(("problems", "id"), Order::Asc)]); // tiebreaker

            query.to_string(SqliteQueryBuilder)
        };
        println!("{query}");

        let problems: Vec<ProblemSummary> = sqlx::query_as(AssertSqlSafe(query))
            .fetch_all(&st.db_pool)
            .await?
            .into_iter()
            .map(|r: QueryResult| ProblemSummary {
                id: r.id,
                title: r.title,
                difficulty: None,
                problem_type: to_problem_type(r.runtype),
                source: r.source,
                authors: match r.authors {
                    None => vec![],
                    Some(s) => s.split(",").map(|a| a.parse().unwrap()).collect(),
                },
                solves: r.solves,
                status: match r.max_score {
                    Some(ms) => ProblemStatus::Attempted(ms),
                    None => ProblemStatus::Unattempted,
                },
                created_at: r.created_at,
            })
            .collect();
        Ok(Json(ProblemCollectionPage { items: problems }))
    }

    #[utoipa::path(
        get,
        path = super::PROBLEMS_ID_URI,
        responses(
            (status = OK, body = ProblemDetails),
            (status = NOT_FOUND, body = String)
        ),
    )]
    pub async fn problems_id(
        Extension(st): Extension<Arc<app::State>>,
        auth: auth::AuthSession,
        Path(problem_id): Path<i64>,
    ) -> ApiResult<Json<super::ProblemDetails>> {
        let user = auth.user().await;

        let problem = sqlx::query!(
            r#"
            SELECT id, title, source, tl, ml, runtype, created_at
            FROM problems
            WHERE id = ?
            "#,
            problem_id
        )
        .fetch_optional(&st.db_pool)
        .await?
        .context_not_found(format!("No such problem: {problem_id}"))?;

        let subtasks = sqlx::query_scalar!(
            r#"
            SELECT score
            FROM subtasks
            WHERE FK_problems_id = ?
            ORDER BY subtask ASC
            "#,
            problem_id
        )
        .fetch_all(&st.db_pool)
        .await?;

        let max_score = match user {
            None => Some(0),
            Some(u) => {
                sqlx::query_scalar!(
                    r#"
                SELECT MAX(score)
                FROM submissions
                WHERE FK_problems_id = ? AND FK_users_id = ?
                "#,
                    problem_id,
                    u.id
                )
                .fetch_one(&st.db_pool)
                .await?
            }
        };

        let authors = sqlx::query_scalar!(
            r#"
            SELECT FK_users_id
            FROM problem_authors
            WHERE FK_problems_id = ?
            "#,
            problem_id
        )
        .fetch_all(&st.db_pool)
        .await?;

        let num_solves = sqlx::query_scalar!(
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
        )
        .fetch_all(&st.db_pool)
        .await
        .iter()
        .len();

        // TODO fetch from R2
        let problem_statement =
            fs::read_to_string(format!("state/problems/{}/statement.html", problem_id))
                .await
                .unwrap_or(String::new());

        let tags = sqlx::query_scalar!(
            r#"
            SELECT
                tag
            FROM
                problem_tags
            WHERE FK_problems_id = ?
            ORDER BY tag
            "#,
            problem_id
        )
        .fetch_all(&st.db_pool)
        .await?;

        let r = super::ProblemDetails {
            summary: ProblemSummary {
                id: problem_id,
                title: problem.title,
                difficulty: None, // TODO
                problem_type: to_problem_type(problem.runtype),
                source: problem.source,
                authors: authors,
                solves: num_solves as i64,
                status: match max_score {
                    Some(s) => ProblemStatus::Attempted(s),
                    None => ProblemStatus::Unattempted,
                },
                created_at: problem.created_at,
            },
            statement_html: problem_statement,
            tags: tags,
            time_limit: problem.tl,
            memory_limit: problem.ml,
            attachments: vec![], // TODO, need to set up R2
            subtasks: subtasks,
            submission_policy: SubmissionPolicy {
                // TODO
                allowed: SubmissionPermitted::Allowed,
                allowed_languages: vec![],
                max_source_bytes: 65536,     // TODO
                cooldown_seconds: 0,         // TODO,
                remaining_submissions: None, // TODO
            },
            editorial: ProblemEditorial::NotPublished, // TODO
        };

        Ok(Json(r))
    }
}
