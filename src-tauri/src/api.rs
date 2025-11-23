use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRequest {
    pub url: String,
    pub method: String,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

/// Выполнение HTTP запроса
#[tauri::command]
pub async fn make_request(request: ApiRequest) -> Result<ApiResponse, String> {
    // TODO: Реализовать HTTP запросы
    Ok(ApiResponse {
        status: 200,
        body: format!("Response from {}", request.url),
        headers: vec![("content-type".to_string(), "application/json".to_string())],
    })
}

/// Получение данных GET запросом
#[tauri::command]
pub async fn fetch_data(url: String) -> Result<String, String> {
    // TODO: Реализовать GET запрос
    Ok(format!("Data from {}", url))
}

/// Отправка данных POST запросом
#[tauri::command]
pub async fn post_data(url: String, data: String) -> Result<ApiResponse, String> {
    // TODO: Реализовать POST запрос
    Ok(ApiResponse {
        status: 201,
        body: format!("Posted to {}: {}", url, data),
        headers: vec![],
    })
}

