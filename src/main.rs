use axum::{
    extract::{Path, State}, 
    http::StatusCode, 
    response::IntoResponse, 
    routing::{get, post},
    Json, Router
};

use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, PgPool, Postgres, QueryBuilder};

#[derive(Deserialize, FromRow, Serialize)]
struct Article {
    title: String,
    content: String,
    published_date: String,
}

impl SQLStatements<usize> for Article {
    // fn insert(&self) -> String {
    //     let mut query_builder: QueryBuilder<Postgres> = 
    //         QueryBuilder::new("INSERT INTO articles (title, content, published_date)");
    //
    //     query_builder.push_values([self], |mut b, article| {
    //         b.push_bind(article.title.clone())
    //             .push_bind(article.content.clone())
    //             .push_bind(article.published_date.clone());
    //     });
    //     query_builder.into_sql()
    // }

    fn insert(&self) -> String {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("INSERT INTO articles (title, content, published_date)");

        query_builder.push_values([self], |mut b, article| {
            b.push_bind(article.title.clone())
                .push_bind(article.content.clone())
                .push_bind(article.published_date.clone());
        });
        query_builder.into_sql()
}


    fn select(key: usize) -> String {
        format!(
            "SELECT title, content, published_date FROM articles WHERE id = {}",
            key
        )
    }

}

fn not_found(e: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::NOT_FOUND,
        format!("Article with id {} not found", e),
    )
}

fn internal_server_error(e: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Error creating article: {}", e),
    )
}

trait SQLStatements<T> {
    fn insert(&self) -> String;
    fn select(key: T) -> String;
}

async fn create_article(
    State(pool): State<PgPool>,
    Json(new_article): Json<Article>,
) -> impl IntoResponse {

    sqlx::query(&new_article.insert())
        .execute(&pool)
        .await
        .map(|_| (StatusCode::OK, "Article created successfully".to_string()))
        .map_err(internal_server_error)
}

async fn get_article(
    Path(article_id): Path<usize>,
    State(pool): State<PgPool>,
) -> Result<Json<Article>, (StatusCode, String)> {

    sqlx::query_as(&Article::select(article_id))
        .fetch_one(&pool)
        .await
        .map(Json)
        .map_err(not_found)
}


#[shuttle_runtime::main]
async fn axum(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://hobo@localhost:5432/postgres"
    )]
    pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(shuttle_runtime::CustomError::new)?;


    let router = Router::new()
        .route("/articles", post(create_article))
        .route("/articles/:id", get(get_article))
        .with_state(pool);

    Ok(router.into())
}
