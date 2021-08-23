use crate::netease::crypto::Crypto;
use lazy_static::lazy_static;
use rand::{thread_rng, Rng};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, COOKIE, REFERER, USER_AGENT};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

static LINUX_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.90 Safari/537.36";
static AGENT: &str =
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:90.0) Gecko/20100101 Firefox/90.0";

lazy_static! {
    static ref _CSRF: Regex = Regex::new(r"_csrf=(?P<csrf>[^(;|$)]+)").unwrap();
    static ref DOMAIN: Regex = Regex::new(r#"\s*Domain=[^(;|$)]+;*"#).unwrap();
}

pub(crate) fn get_timestamp() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    since_the_epoch.to_string()
}

pub(crate) fn get_timestamp_ten() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    String::from(&since_the_epoch.to_string()[..10])
}

pub(crate) async fn request(
    mut url: &str,
    method: &str,
    query_params: HashMap<&str, &str>,
    request_params: HashMap<&str, &str>,
) -> reqwest::Response {
    let crypto = request_params.get("crypto").unwrap_or(&"weapi");

    let mut headers = HeaderMap::new();

    headers.insert("X-Real-IP", "182.201.169.6".parse().unwrap());

    if method.to_uppercase() == "POST" {
        headers.insert(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
    }
    if url.contains("music.163.com") {
        headers.insert(REFERER, "https://music.163.com".parse().unwrap());
    }
    if let Some(cookie) = request_params.get("cookie") {
        headers.insert(COOKIE, cookie.parse().unwrap());
    }

    match crypto {
        &"weapi" => {
            headers.insert(USER_AGENT, AGENT.parse().unwrap());
        }
        &"linuxapi" => {
            headers.insert(USER_AGENT, LINUX_AGENT.parse().unwrap());
        }
        &"eapi" => {
            headers.insert("osver", "".parse().unwrap());
            headers.insert("deviceId", "".parse().unwrap());
            headers.insert("appver", "8.0.0".parse().unwrap());
            headers.insert("appver", "140".parse().unwrap());
            headers.insert("buildver", get_timestamp_ten().parse().unwrap());
            headers.insert("resolution", "1920x1080".parse().unwrap());
            headers.insert("os", "android".parse().unwrap());
            let mut rng = thread_rng();
            headers.insert(
                "requestId",
                format!("{}_{:04}", get_timestamp(), rng.gen_range(0..=9999))
                    .parse()
                    .unwrap(),
            );
        }
        &_ => {}
    }

    let empty_cookie = HeaderValue::from_static("");
    let cookie = headers
        .get(COOKIE)
        .unwrap_or(&empty_cookie)
        .to_str()
        .unwrap();

    let body = match crypto {
        &"weapi" => {
            let csrf_token = if let Some(caps) = _CSRF.captures(cookie) {
                caps.name("csrf").unwrap().as_str()
            } else {
                ""
            };
            let mut params = query_params;
            params.insert("csrf_token", csrf_token);
            log::info!("params: {:?}", &params);
            Crypto::we_api(&serde_json::to_string(&params).unwrap())
        }
        &"linuxapi" => {
            let data = format!(
                r#"{{"method":"{}","url":"{}","params":{}}}"#,
                method,
                url.replace("weapi", "api"),
                serde_json::to_string(&query_params).expect("failed to serialize query params")
            );
            url = "https://music.163.com/api/linux/forward";
            log::info!("data: {}", &data);
            Crypto::linux_api(&data)
        }
        &"eapi" => {
            let params = query_params;
            url = "https://interface3.music.163.com/eapi/song/enhance/player/url";
            Crypto::e_api(url, &serde_json::to_string(&params).unwrap())
        }
        _ => String::from(""),
    };

    log::debug!("body: {}", &body);

    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()
        .unwrap();

    let response = if crypto == &"eapi" {
        let resp = client.post(url).body(body).send().await;
        log::debug!("{:?}", &resp);
        resp.unwrap()
    } else {
        client.post(url).body(body).send().await.unwrap()
    };

    log::debug!("response: {:?}", &response);

    response
}

#[cfg(test)]
mod test {
    use crate::netease::request::{get_timestamp, get_timestamp_ten};

    #[test]
    fn test_time() {
        log::info!("{}", get_timestamp());
        log::info!("{}", get_timestamp_ten());
    }
}
