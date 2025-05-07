use std::io::{self, Write as _};

use google_ai_rs::{Auth, Client};

#[tokio::main]
async fn main() {
	// let api_key = std::env::var("GEMINI_API_KEY").unwrap();
	// let client = Client::new(api_key.into()).await.unwrap();
	let auth = Auth::service_account("/home/victor/GolandProjects/API/protobuf/google_ai/genai-key.json").await.unwrap();
	let client = Client::new(auth).await.unwrap();
	let m = client.generative_model("gemini-1.5-pro");
	let mut resp = m.stream_generate_content("How many seconds moved?").await.unwrap();
	resp.write_to(&mut io::stdout()).await.unwrap();
	// io::stdout().write(&resp.try_into_bytes().unwrap()).unwrap();
}

