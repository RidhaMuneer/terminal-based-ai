use dotenv::dotenv;
use hyper::client::HttpConnector;
use hyper::{header, Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde_derive::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::env;
use std::io::{stdin, stdout, Write};

#[derive(Deserialize, Debug)]
struct AIResponse {
    candidates: Vec<Candidate>,
    usageMetadata: UsageMetadata,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: Content,
    finishReason: String,
    index: i32,
    safetyRatings: Vec<SafetyRating>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Content {
    parts: Vec<Part>,
    role: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Part {
    text: String,
}

#[derive(Deserialize, Debug)]
struct SafetyRating {
    category: String,
    probability: String,
}

#[derive(Deserialize, Debug)]
struct UsageMetadata {
    promptTokenCount: i32,
    candidatesTokenCount: i32,
    totalTokenCount: i32,
}

#[derive(Serialize, Debug)]
struct GenerateContentRequest {
    contents: Vec<Content>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok(); // Load environment variables from .env file

    let https = HttpsConnector::new();
    let client: Client<HttpsConnector<HttpConnector>> = Client::builder().build(https);
    let uri = "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent";
    let ai_token = match env::var("AI_API_KEYS") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("Error: AI_API_KEYS environment variable is not set");
            return Ok(());
        }
    };
    let request_url = format!("{}?key={}", uri, ai_token);

    loop {
        print!("> ");
        stdout().flush().unwrap();
        let mut user_text = String::new();

        stdin()
            .read_line(&mut user_text)
            .expect("Failed to read line");

        print!("");

        let sp = Spinner::new(&Spinners::Dots9, "\t\tI'm thinking".into());

        let ai_request = GenerateContentRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: user_text.trim().to_string(),
                }],
                role: "user".to_string(),
            }],
        };

        let json_body = match serde_json::to_vec(&ai_request) {
            Ok(body) => body,
            Err(e) => {
                eprintln!("Error serializing request: {:?}", e);
                sp.stop();
                continue;
            }
        };

        let body = Body::from(json_body);

        let req = match Request::post(&request_url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
        {
            Ok(request) => request,
            Err(e) => {
                eprintln!("Error building request: {:?}", e);
                sp.stop();
                continue;
            }
        };

        let res = match client.request(req).await {
            Ok(response) => response,
            Err(e) => {
                eprintln!("Error sending request: {:?}", e);
                sp.stop();
                continue;
            }
        };

        let body_bytes = match hyper::body::to_bytes(res.into_body()).await {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Error reading response body: {:?}", e);
                sp.stop();
                continue;
            }
        };

        let response: AIResponse = match serde_json::from_slice(&body_bytes) {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Error deserializing response: {:?}", e);
                sp.stop();
                continue;
            }
        };

        sp.stop();
        print!("\n");
        print!(
            "{}",
            format_json(&response.candidates[0].content.parts[0].text.to_string())
        );
        print!("\n");
    }
    Ok(())
}

fn format_json(json_content: &str) -> String {
    let formatted_content = json_content.replace("```", "'''");

    formatted_content
}
