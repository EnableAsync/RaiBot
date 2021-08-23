use crate::netease::request::request;
use reqwest::Response;
use std::collections::HashMap;

pub(crate) async fn search(name: &str) -> Response {
    let url = "https://music.163.com/weapi/search/get";
    let mut query_params: HashMap<&str, &str> = HashMap::new();
    query_params.insert("s", name);
    query_params.insert("type", "1");
    query_params.insert("limit", "1");
    query_params.insert("offset", "0");

    request(url, "POST", query_params, HashMap::new()).await
}

pub(crate) async fn song_url(id: &str) -> Response {
    let url = "https://music.163.com/api/song/enhance/player/url";
    let mut query_params: HashMap<&str, &str> = HashMap::new();
    let mut ids = String::from("[");
    ids.push_str(id);
    ids.push_str("]");
    query_params.insert("ids", &ids);
    query_params.insert("br", "320000");
    let mut request_params: HashMap<&str, &str> = HashMap::new();
    request_params.insert("crypto", "linuxapi");
    request_params.insert("cookie", ";os=pc;");
    request(url, "POST", query_params, request_params).await
}

#[cfg(test)]
mod tests {
    use crate::netease::api::{search, song_url};
    use serde_json::Value;

    #[tokio::test]
    async fn test_search() {
        let name = "Flame of Nuclear";

        let response = search(name).await.text().await.unwrap();
        log::info!("{}", &response);

        let value: Value = serde_json::from_str(&response).expect("failed to deserialize json");
        let id = &value["result"]["songs"][0]["id"];
        let artist = &value["result"]["songs"][0]["artists"][0]["name"];
        log::debug!("id: {}, name: {} artist: {}", id, name, artist);

        let response = song_url(&id.to_string()).await.text().await.unwrap();
        log::info!("{}", response);
    }
}
