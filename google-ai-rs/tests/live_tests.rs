use google_ai_rs::{
    genai::{GenerativeModel, Info},
    AsSchema, Client,
};

use serde::Deserialize;
use std::{collections::HashSet, env, error::Error, io};
use tokio::sync::OnceCell;

const DEFAULT_MODEL: &str = "gemini-1.5-flash";

static CLIENT: OnceCell<Result<Client>> = OnceCell::const_new();

#[tokio::test]
#[ignore = "Requires API access"]
async fn basic_generation() -> Result<()> {
    let response = default_model_instance()
        .await?
        .generate_content("What is the average size of a swallow?")
        .await?;

    assert!(!response.text().is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "Requires API access"]
async fn schema() -> Result<()> {
    #[allow(dead_code)]
    #[derive(AsSchema, Deserialize)]
    #[schema(description = "A primary colour")]
    struct PrimaryColor {
        #[schema(description = "The name of the colour")]
        name: String,
        #[schema(description = "The RGB value of the color, in hex")]
        #[serde(rename = "RGB")]
        rgb: String,
    }

    let mut model = get_client()
        .await?
        .generative_model("gemini-1.5-pro-latest")
        .as_response_schema::<Vec<PrimaryColor>>();

    model.set_temperature(0.0);

    let response = model.generate_content("List the primary colors.").await?;
    serde_json::from_slice::<Vec<PrimaryColor>>(&response.into_bytes())?;
    Ok(())
}

#[tokio::test]
#[ignore = "Requires API access"]
async fn streaming() -> Result<()> {
    let mut stream = default_model_instance()
        .await?
        .stream_generate_content("Are you hungry?")
        .await?;
    let mut output = Vec::new();

    stream.write_to(&mut output).await?;

    assert!(!output.is_empty());

    Ok(())
}

#[tokio::test]
#[ignore = "Requires API access"]
async fn chat() -> Result<()> {
    let model = default_model_instance().await?;
    let mut session = model.start_chat();

    let first_response = session
        .send_message("Name the 5 most popular puppy breeds.")
        .await?;
    // Check that two items, the sent message and the response) were
    // added to the history.
    assert_eq!(
        session.history.len(),
        2,
        "history length: got {} want 2",
        session.history.len()
    );

    assert!(!first_response.text().is_empty());
    // Last history item is the one we just got from the model.

    let mut second_response = session.stream_send_message("Which is best?").await?;
    let total = second_response.write_to(&mut io::sink()).await?;

    assert_eq!(
        session.history.len(),
        4,
        "history length: got {} want 4",
        session.history.len()
    );

    assert!(total > 0);
    Ok(())
}

#[tokio::test]
#[ignore = "Requires API access"]
async fn embeddings() -> Result<()> {
    let model = get_client().await?.embedding_model("embedding-001");

    // Single embedding
    let response = model.embed_content("cheddar cheese").await?;
    assert!(response.embedding.is_some());

    // Batch embeddings
    let batch = model
        .new_batch()
        .add_content("cheddar cheese")
        .add_content_with_title("Cheese Report", "I love cheddar cheese.");

    let batch_response = batch.embed().await?;
    assert_eq!(batch_response.embeddings.len(), 2);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires API access"]
async fn model_info() -> Result<()> {
    let model = default_model_instance().await?;

    match model.info().await? {
        Info::Tuned(_) => return Err("shouldn't get tuned model info".into()),
        Info::Model(info) => assert_eq!(&info.name, model.full_name()),
    };

    Ok(())
}

#[tokio::test]
#[ignore = "Requires API access"]
async fn models() -> Result<()> {
    let mut models = get_client().await?.list_models().await;
    let mut want = HashSet::from(["models/gemini-1.5-pro", "models/embedding-001"]);

    loop {
        if want.is_empty() {
            return Ok(());
        }

        if let Some(model) = models.next().await? {
            if want.contains(model.name.as_str()) {
                want.remove(model.name.as_str());
            }
        } else {
            break;
        }
    }

    if !want.is_empty() {
        return Err(format!("missing expected model(s): {want:#?}").into());
    }

    return Ok(());
}

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

async fn default_model_instance() -> Result<GenerativeModel<'static>> {
    let mut model = get_client().await?.generative_model(DEFAULT_MODEL);
    model.set_temperature(0.0);
    Ok(model)
}

async fn get_client() -> Result<&'static Client> {
    match CLIENT.get_or_init(initiate_client).await {
        Ok(t) => Ok(t),
        Err(err) => Err(err.to_string().into()),
    }
}

async fn initiate_client() -> Result<Client> {
    let api_key =
        env::var("GEMINI_API_KEY").map_err(|_| "GEMINI_API_KEY should be set for live test")?;
    Ok(Client::new(api_key.into()).await?)
}
