use crate::{domain, model::TestRun, store};
use lambda_http::{http::StatusCode, IntoResponse, Request, RequestExt, Response};
use serde_json::json;
use tracing::{error, info, instrument, warn};

type E = Box<dyn std::error::Error + Sync + Send + 'static>;

/// Get a TestRun
#[instrument(skip(store))]
pub async fn get_testrun(
    store: &dyn store::StoreGet,
    event: Request,
) -> Result<impl IntoResponse, E> {
    let path_parameters = event.path_parameters();
    let id = match path_parameters.first("id") {
        Some(id) => id,
        None => {
            warn!("Missing 'id' parameter in path");
            return Ok(response(
                StatusCode::BAD_REQUEST,
                json!({ "message": "Missing 'id' parameter in path" }).to_string(),
            ));
        }
    };

    info!("Fetching Test Run #{}", id);
    let testrun = domain::testrun::get_testrun(store, id).await;

    Ok(match testrun {
        // TestRun exists
        Ok(Some(testrun)) => response(StatusCode::OK, json!(testrun).to_string()),
        // TestRun doesn't exist
        Ok(None) => {
            warn!("TestRun not found: {}", id);
            response(
                StatusCode::NOT_FOUND,
                json!({"message": "TestRun not found"}).to_string(),
            )
        }
        // Error
        Err(err) => {
            error!("Error fetching testrun: {}", err);
            response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"message": "Error fetching testrun"}).to_string(),
            )
        }
    })
}

/// Put a TestRun
#[instrument(skip(store))]
pub async fn put_testrun(
    store: &dyn store::StorePut,
    event: Request,
) -> Result<impl IntoResponse, E> {
    let path_parameters = event.path_parameters();
    let id = match path_parameters.first("id") {
        Some(id) => id,
        None => {
            warn!("Missing 'id' parameter in path");
            return Ok(response(
                StatusCode::BAD_REQUEST,
                json!({ "message": "Missing 'id' parameter in path" }).to_string(),
            ));
        }
    };

    let testrun: TestRun = match event.payload() {
        Ok(Some(testrun)) => testrun,
        Ok(None) => {
            warn!("Missing testrun in request body");
            return Ok(response(
                StatusCode::BAD_REQUEST,
                json!({"message": "Missing testrun in request body"}).to_string(),
            ));
        }
        Err(err) => {
            warn!("Failed to parse testrun from request body: {}", err);
            return Ok(response(
                StatusCode::BAD_REQUEST,
                json!({"message": "Failed to parse testrun from request body"}).to_string(),
            ));
        }
    };

    info!("Parsed testrun: {:?}", testrun);

    if testrun.id != id {
        warn!(
            "TestRun ID in path ({}) does not match ID in body ({})",
            id, testrun.id
        );
        return Ok(response(
            StatusCode::BAD_REQUEST,
            json!({"message": "TestRun ID in path does not match ID in body"}).to_string(),
        ));
    }

    // Put testrun
    let res = domain::testrun::put_testrun(store, &testrun).await;

    // Return response
    //
    // If the put was successful, we return a 201 Created. Otherwise, we return
    // a 500 Internal Server Error.
    Ok(match res {
        // Testrun created
        Ok(_) => {
            info!("Queued testrun {:?}", testrun.id);
            response(
                StatusCode::CREATED,
                json!({"message": "Testrun queued"}).to_string(),
            )
        }
        // Error creating testrun
        Err(err) => {
            error!("Failed to create testrun {}: {}", testrun.id, err);
            response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"message": "Failed to create testrun"}).to_string(),
            )
        }
    })
}

/// HTTP Response with a JSON payload
fn response(status_code: StatusCode, body: String) -> Response<String> {
    Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap()
}
