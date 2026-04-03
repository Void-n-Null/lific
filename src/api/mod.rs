use axum::{
    Router,
    extract::{Json, Path, Query, State},
    routing::{delete, get, post, put},
};
use tower_http::cors::CorsLayer;

use crate::db::{DbPool, models::*, queries};
use crate::error::LificError;

/// Build the full API router.
pub fn router(db: DbPool) -> Router {
    Router::new()
        // Projects
        .route("/api/projects", get(list_projects).post(create_project))
        .route(
            "/api/projects/{id}",
            get(get_project)
                .put(update_project)
                .delete(delete_project_handler),
        )
        // Issues
        .route("/api/issues", get(list_issues).post(create_issue))
        .route(
            "/api/issues/{id}",
            get(get_issue)
                .put(update_issue)
                .delete(delete_issue_handler),
        )
        .route("/api/issues/resolve/{identifier}", get(resolve_issue))
        // Issue relations
        .route("/api/issues/link", post(link_issues))
        .route("/api/issues/unlink", post(unlink_issues))
        // Modules
        .route("/api/modules", get(list_modules).post(create_module))
        .route(
            "/api/modules/{id}",
            put(update_module).delete(delete_module_handler),
        )
        // Labels
        .route("/api/labels", get(list_labels).post(create_label))
        .route("/api/labels/{id}", delete(delete_label_handler))
        // Pages
        .route("/api/pages", get(list_pages_handler).post(create_page))
        .route(
            "/api/pages/{id}",
            get(get_page).put(update_page).delete(delete_page_handler),
        )
        // Folders
        .route(
            "/api/folders",
            get(list_folders_handler).post(create_folder),
        )
        .route("/api/folders/{id}", delete(delete_folder_handler))
        // Search
        .route("/api/search", get(search))
        // Board view
        .route("/api/projects/{id}/board", get(get_board))
        // Health
        .route("/api/health", get(health))
        .layer(CorsLayer::permissive())
        .with_state(db)
}

/// Execute a read-only operation against the read pool.
fn with_read<F, T>(db: &DbPool, f: F) -> Result<T, LificError>
where
    F: FnOnce(&rusqlite::Connection) -> Result<T, LificError>,
{
    let conn = db.read()?;
    f(&conn)
}

/// Execute a write operation against the exclusive write connection.
fn with_write<F, T>(db: &DbPool, f: F) -> Result<T, LificError>
where
    F: FnOnce(&rusqlite::Connection) -> Result<T, LificError>,
{
    let conn = db.write()?;
    f(&conn)
}

async fn health() -> &'static str {
    "ok"
}

async fn list_projects(State(db): State<DbPool>) -> Result<Json<Vec<Project>>, LificError> {
    with_read(&db, queries::list_projects).map(Json)
}

async fn get_project(
    State(db): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Json<Project>, LificError> {
    with_read(&db, |conn| queries::get_project(conn, id)).map(Json)
}

async fn create_project(
    State(db): State<DbPool>,
    Json(input): Json<CreateProject>,
) -> Result<Json<Project>, LificError> {
    with_write(&db, |conn| queries::create_project(conn, &input)).map(Json)
}

async fn update_project(
    State(db): State<DbPool>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateProject>,
) -> Result<Json<Project>, LificError> {
    with_write(&db, |conn| queries::update_project(conn, id, &input)).map(Json)
}

async fn delete_project_handler(
    State(db): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, LificError> {
    with_write(&db, |conn| queries::delete_project(conn, id))?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

async fn list_issues(
    State(db): State<DbPool>,
    Query(q): Query<ListIssuesQuery>,
) -> Result<Json<Vec<Issue>>, LificError> {
    with_read(&db, |conn| queries::list_issues(conn, &q)).map(Json)
}

async fn get_issue(
    State(db): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Json<Issue>, LificError> {
    with_read(&db, |conn| queries::get_issue(conn, id)).map(Json)
}

async fn resolve_issue(
    State(db): State<DbPool>,
    Path(identifier): Path<String>,
) -> Result<Json<Issue>, LificError> {
    with_read(&db, |conn| {
        let id = queries::resolve_identifier(conn, &identifier)?;
        queries::get_issue(conn, id)
    })
    .map(Json)
}

async fn create_issue(
    State(db): State<DbPool>,
    Json(input): Json<CreateIssue>,
) -> Result<Json<Issue>, LificError> {
    with_write(&db, |conn| queries::create_issue(conn, &input)).map(Json)
}

async fn update_issue(
    State(db): State<DbPool>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateIssue>,
) -> Result<Json<Issue>, LificError> {
    with_write(&db, |conn| queries::update_issue(conn, id, &input)).map(Json)
}

async fn delete_issue_handler(
    State(db): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, LificError> {
    with_write(&db, |conn| queries::delete_issue(conn, id))?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

#[derive(serde::Deserialize)]
struct LinkRequest {
    source: String,
    target: String,
    relation_type: String,
}

#[derive(serde::Deserialize)]
struct UnlinkRequest {
    source: String,
    target: String,
}

async fn link_issues(
    State(db): State<DbPool>,
    Json(input): Json<LinkRequest>,
) -> Result<Json<serde_json::Value>, LificError> {
    with_write(&db, |conn| {
        let source_id = queries::resolve_identifier(conn, &input.source)?;
        let target_id = queries::resolve_identifier(conn, &input.target)?;
        queries::link_issues(conn, source_id, target_id, &input.relation_type)
    })?;
    Ok(Json(serde_json::json!({"linked": true})))
}

async fn unlink_issues(
    State(db): State<DbPool>,
    Json(input): Json<UnlinkRequest>,
) -> Result<Json<serde_json::Value>, LificError> {
    with_write(&db, |conn| {
        let source_id = queries::resolve_identifier(conn, &input.source)?;
        let target_id = queries::resolve_identifier(conn, &input.target)?;
        queries::unlink_issues(conn, source_id, target_id)
    })?;
    Ok(Json(serde_json::json!({"unlinked": true})))
}

#[derive(serde::Deserialize)]
struct ModuleQuery {
    project_id: i64,
}

async fn list_modules(
    State(db): State<DbPool>,
    Query(q): Query<ModuleQuery>,
) -> Result<Json<Vec<Module>>, LificError> {
    with_read(&db, |conn| queries::list_modules(conn, q.project_id)).map(Json)
}

async fn create_module(
    State(db): State<DbPool>,
    Json(input): Json<CreateModule>,
) -> Result<Json<Module>, LificError> {
    with_write(&db, |conn| queries::create_module(conn, &input)).map(Json)
}

async fn update_module(
    State(db): State<DbPool>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateModule>,
) -> Result<Json<Module>, LificError> {
    with_write(&db, |conn| queries::update_module(conn, id, &input)).map(Json)
}

async fn delete_module_handler(
    State(db): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, LificError> {
    with_write(&db, |conn| queries::delete_module(conn, id))?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

#[derive(serde::Deserialize)]
struct LabelQuery {
    project_id: i64,
}

async fn list_labels(
    State(db): State<DbPool>,
    Query(q): Query<LabelQuery>,
) -> Result<Json<Vec<Label>>, LificError> {
    with_read(&db, |conn| queries::list_labels(conn, q.project_id)).map(Json)
}

async fn create_label(
    State(db): State<DbPool>,
    Json(input): Json<CreateLabel>,
) -> Result<Json<Label>, LificError> {
    with_write(&db, |conn| queries::create_label(conn, &input)).map(Json)
}

async fn delete_label_handler(
    State(db): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, LificError> {
    with_write(&db, |conn| queries::delete_label(conn, id))?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

#[derive(serde::Deserialize)]
struct PageQuery {
    project_id: Option<i64>,
    folder_id: Option<i64>,
}

async fn list_pages_handler(
    State(db): State<DbPool>,
    Query(q): Query<PageQuery>,
) -> Result<Json<Vec<Page>>, LificError> {
    with_read(&db, |conn| {
        queries::list_pages(conn, q.project_id, q.folder_id)
    })
    .map(Json)
}

async fn get_page(State(db): State<DbPool>, Path(id): Path<i64>) -> Result<Json<Page>, LificError> {
    with_read(&db, |conn| queries::get_page(conn, id)).map(Json)
}

async fn create_page(
    State(db): State<DbPool>,
    Json(input): Json<CreatePage>,
) -> Result<Json<Page>, LificError> {
    with_write(&db, |conn| queries::create_page(conn, &input)).map(Json)
}

async fn update_page(
    State(db): State<DbPool>,
    Path(id): Path<i64>,
    Json(input): Json<UpdatePage>,
) -> Result<Json<Page>, LificError> {
    with_write(&db, |conn| queries::update_page(conn, id, &input)).map(Json)
}

async fn delete_page_handler(
    State(db): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, LificError> {
    with_write(&db, |conn| queries::delete_page(conn, id))?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

#[derive(serde::Deserialize)]
struct FolderQuery {
    project_id: i64,
}

async fn list_folders_handler(
    State(db): State<DbPool>,
    Query(q): Query<FolderQuery>,
) -> Result<Json<Vec<Folder>>, LificError> {
    with_read(&db, |conn| queries::list_folders(conn, q.project_id)).map(Json)
}

async fn create_folder(
    State(db): State<DbPool>,
    Json(input): Json<CreateFolder>,
) -> Result<Json<Folder>, LificError> {
    with_write(&db, |conn| queries::create_folder(conn, &input)).map(Json)
}

async fn delete_folder_handler(
    State(db): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, LificError> {
    with_write(&db, |conn| queries::delete_folder(conn, id))?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

async fn search(
    State(db): State<DbPool>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, LificError> {
    with_read(&db, |conn| queries::search(conn, &q)).map(Json)
}

#[derive(serde::Deserialize)]
struct BoardQuery {
    #[serde(default = "default_group_by")]
    group_by: String,
}

fn default_group_by() -> String {
    "status".to_string()
}

async fn get_board(
    State(db): State<DbPool>,
    Path(project_id): Path<i64>,
    Query(q): Query<BoardQuery>,
) -> Result<Json<serde_json::Value>, LificError> {
    let issues = with_read(&db, |conn| {
        queries::list_issues(
            conn,
            &ListIssuesQuery {
                project_id: Some(project_id),
                status: None,
                priority: None,
                module_id: None,
                label: None,
                workable: None,
                limit: Some(500),
                offset: None,
            },
        )
    })?;

    let mut board: std::collections::BTreeMap<String, Vec<&Issue>> =
        std::collections::BTreeMap::new();
    for issue in &issues {
        let key = match q.group_by.as_str() {
            "priority" => issue.priority.clone(),
            "module" => issue
                .module_id
                .map(|_| "has_module".to_string())
                .unwrap_or("unassigned".to_string()),
            _ => issue.status.clone(),
        };
        board.entry(key).or_default().push(issue);
    }

    Ok(Json(serde_json::json!(board)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_app() -> Router {
        let db = crate::db::open_memory().expect("test db");
        router(db)
    }

    /// Seed a project and return its id.
    async fn seed_project(app: &Router) -> (i64, serde_json::Value) {
        let body = serde_json::json!({
            "name": "Test Project",
            "identifier": "TST",
            "description": "integration test project"
        });
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/projects")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let val: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let id = val["id"].as_i64().unwrap();
        (id, val)
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn project_crud_lifecycle() {
        let app = test_app();

        // Create
        let (id, project) = seed_project(&app).await;
        assert_eq!(project["identifier"], "TST");

        // Get
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/projects/{id}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // List
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/projects")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let list: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(list.len(), 1);

        // Update
        let update = serde_json::json!({"name": "Renamed"});
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/projects/{id}"))
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&update).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let updated: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(updated["name"], "Renamed");
        assert_eq!(updated["identifier"], "TST"); // unchanged

        // Delete
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/projects/{id}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify gone
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/projects/{id}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn issue_crud_lifecycle() {
        let app = test_app();
        let (project_id, _) = seed_project(&app).await;

        // Create issue
        let body = serde_json::json!({
            "project_id": project_id,
            "title": "Fix the bug",
            "status": "todo",
            "priority": "high"
        });
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/issues")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let issue: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let issue_id = issue["id"].as_i64().unwrap();
        assert_eq!(issue["identifier"], "TST-1");
        assert_eq!(issue["priority"], "high");

        // List with filter
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/issues?project_id={project_id}&status=todo"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let list: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(list.len(), 1);

        // Update
        let update = serde_json::json!({"status": "active"});
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/issues/{issue_id}"))
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&update).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let updated: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(updated["status"], "active");

        // Resolve by identifier
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/issues/resolve/TST-1")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Delete
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/issues/{issue_id}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn search_returns_results() {
        let app = test_app();
        let (project_id, _) = seed_project(&app).await;

        // Create an issue to search for
        let body = serde_json::json!({
            "project_id": project_id,
            "title": "Unique searchable title xyz"
        });
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/issues")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/search?query=searchable")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let results: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn get_nonexistent_project_returns_404() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/projects/99999")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn board_groups_by_status() {
        let app = test_app();
        let (project_id, _) = seed_project(&app).await;

        for (title, status) in [("A", "todo"), ("B", "active"), ("C", "todo")] {
            let body = serde_json::json!({
                "project_id": project_id,
                "title": title,
                "status": status
            });
            app.clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/issues")
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                        .unwrap(),
                )
                .await
                .unwrap();
        }

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/projects/{project_id}/board"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let board: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(board["todo"].as_array().unwrap().len(), 2);
        assert_eq!(board["active"].as_array().unwrap().len(), 1);
    }
}
