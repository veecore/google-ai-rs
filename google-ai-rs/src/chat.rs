use std::{collections::HashMap, io::Write};

use crate::{
    content::IntoContents,
    error::{ActionError, Error, ServiceError},
    genai::{GenerativeModel, ResponseStream as GenResponseStream},
    proto::{part::Data, Candidate, CitationMetadata, Content, GenerateContentResponse, Part},
};

/// Interactive chat session maintaining conversation history
///
/// # Example
/// ```
/// # use google_ai_rs::{Client, GenerativeModel};
/// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
/// # let auth = "YOUR-API-KEY".into();
/// let client = Client::new(auth).await?;
/// let model = client.generative_model("gemini-1.5-pro");
/// let mut chat = model.start_chat();
/// let response = chat.send_message("Hello!").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Session<'m> {
    model: &'m GenerativeModel<'m>,
    pub history: Vec<Content>,
}

impl GenerativeModel<'_> {
    /// Starts a new chat session with empty history
    pub fn start_chat(&self) -> Session<'_> {
        Session {
            model: self,
            history: Vec::new(),
        }
    }
}

impl<'m> Session<'m> {
    /// Sends a message and appends response to history
    ///
    /// # Errors
    /// Returns [`Error::Service`] if no valid candidates in response
    pub async fn send_message<T>(&mut self, contents: T) -> Result<GenerateContentResponse, Error>
    where
        T: IntoContents,
    {
        self.history.extend(contents.into_contents());

        let response = self.model.generate_content(self.history.clone()).await?;

        self.add_best_candidate_to_history(&response.candidates)
            .ok_or(Error::Service(ServiceError::InvalidResponse(
                "No valid candidates".into(),
            )))?;

        Ok(response)
    }

    /// Starts a streaming response while maintaining session state
    ///
    /// `NOTE`: history is only added if whole message is consumed
    pub async fn stream_send_message<'s, T>(
        &'s mut self,
        contents: T,
    ) -> Result<ResponseStream<'s, 'm>, Error>
    where
        T: IntoContents,
    {
        self.history.extend(contents.into_contents());

        let stream = self
            .model
            .stream_generate_content(self.history.clone())
            .await?;

        Ok(ResponseStream {
            inner: stream,
            merged_candidates: Vec::new(),
            session: self,
            is_complete: false,
        })
    }

    /// Adds the most appropriate candidate to chat history
    fn add_best_candidate_to_history(&mut self, candidates: &[Candidate]) -> Option<()> {
        candidates.first().and_then(|candidate| {
            candidate.content.as_ref().map(|content| {
                let mut model_content = content.clone();
                model_content.role = "model".to_owned();
                self.history.push(model_content);
            })
        })
    }
}

/// Streaming response handler that maintains session continuity
pub struct ResponseStream<'s, 'm> {
    session: &'s mut Session<'m>,
    inner: GenResponseStream,
    merged_candidates: Vec<Candidate>,
    is_complete: bool,
}

impl<'s, 'm> ResponseStream<'s, 'm> {
    /// Streams content chunks to any `Write` implementer
    ///
    /// # Returns
    /// Total bytes written
    pub async fn write_to<W: Write>(&mut self, writer: &mut W) -> Result<usize, Error> {
        let mut total = 0;

        while let Some(response) = self
            .next()
            .await
            .map_err(|e| Error::Stream(ActionError::Error(e.into())))?
        {
            let bytes = response.try_into_bytes()?;
            let written = writer
                .write(&bytes)
                .map_err(|e| Error::Stream(ActionError::Action(e)))?;
            total += written;
        }

        Ok(total)
    }

    /// Retrieves next chunk of streaming response
    pub async fn next(&mut self) -> Result<Option<GenerateContentResponse>, Error> {
        if self.is_complete {
            return Ok(None);
        }

        match self.inner.next().await? {
            Some(response) => {
                merge_candidates(&mut self.merged_candidates, &response.candidates);
                Ok(Some(response))
            }
            None => {
                self.session
                    .add_best_candidate_to_history(&self.merged_candidates);
                self.is_complete = true;
                Ok(None)
            }
        }
    }
}

/// Merges candidate lists from multiple response chunks
pub fn merge_candidates(merged: &mut Vec<Candidate>, new_candidates: &[Candidate]) {
    if merged.is_empty() {
        merged.extend_from_slice(new_candidates);
        return;
    }

    let candidate_map: HashMap<_, _> = new_candidates
        .iter()
        .filter_map(|c| c.index.as_ref().map(|i| (i, c)))
        .collect();

    for candidate in merged.iter_mut() {
        if let Some(index) = &candidate.index {
            if let Some(new_candidate) = candidate_map.get(index) {
                merge_candidate_data(candidate, new_candidate);
            }
        }
    }
}

/// Merges candidate content and metadata
pub fn merge_candidate_data(target: &mut Candidate, source: &Candidate) {
    // Merge content parts
    if let Some(source_content) = &source.content {
        target.content = match target.content.take() {
            Some(existing) => Some(merge_content(existing, source_content.clone())),
            None => Some(source_content.clone()),
        };
    }

    // Update metadata
    target.finish_reason.clone_from(&source.finish_reason);
    target.safety_ratings.clone_from(&source.safety_ratings);

    // Merge citations
    if let Some(source_citations) = &source.citation_metadata {
        target.citation_metadata = match target.citation_metadata.take() {
            Some(existing) => Some(merge_citations(existing, source_citations)),
            None => Some(source_citations.clone()),
        };
    }
}

/// Merges content parts while combining consecutive text elements
pub fn merge_content(mut existing: Content, update: Content) -> Content {
    existing.parts = merge_parts(existing.parts, update.parts);
    existing
}

/// combines parts while merging adjacent text blocks
pub fn merge_parts(mut existing: Vec<Part>, update: Vec<Part>) -> Vec<Part> {
    let mut buffer = String::new();
    let mut merged = Vec::new();

    // Process existing parts
    for part in existing.drain(..) {
        if let Some(Data::Text(text)) = &part.data {
            buffer.push_str(text);
        } else {
            if !buffer.is_empty() {
                merged.push(Part::text(&buffer));
                buffer.clear();
            }
            merged.push(part);
        }
    }

    // Process new parts
    for part in update {
        if let Some(Data::Text(text)) = &part.data {
            buffer.push_str(text);
        } else {
            if !buffer.is_empty() {
                merged.push(Part::text(&buffer));
                buffer.clear();
            }
            merged.push(part);
        }
    }

    // Add remaining text
    if !buffer.is_empty() {
        merged.push(Part {
            data: Some(Data::Text(buffer)),
        });
    }

    merged
}

/// Combines citation metadata from multiple responses
fn merge_citations(mut existing: CitationMetadata, update: &CitationMetadata) -> CitationMetadata {
    existing
        .citation_sources
        .extend(update.citation_sources.iter().cloned());
    existing
}

#[cfg(test)]
mod tests {
    use super::{merge_candidates, merge_parts};
    use crate::{
        content::IntoParts,
        proto::{Candidate, Content, Part},
    };

    impl Content {
        fn model<P>(parts: P) -> Self
        where
            P: IntoParts,
        {
            Self {
                role: "model".into(),
                parts: parts.into_parts(),
            }
        }
    }

    #[test]
    fn _merge_candidates() {
        let mut c1 = vec![
            Candidate {
                index: Some(2),
                content: Some(Content::model("r1 i2")),
                finish_reason: 1,
                safety_ratings: vec![],
                citation_metadata: None,
                token_count: 0,
                grounding_attributions: vec![],
                grounding_metadata: None,
                avg_logprobs: 0.0,
                logprobs_result: None,
            },
            Candidate {
                index: Some(0),
                content: Some(Content::model("r1 i0")),
                finish_reason: 2,
                safety_ratings: vec![],
                citation_metadata: None,
                token_count: 0,
                grounding_attributions: vec![],
                grounding_metadata: None,
                avg_logprobs: 0.0,
                logprobs_result: None,
            },
        ];

        let c2 = vec![
            Candidate {
                index: Some(0),
                content: Some(Content::model(";r2 i0")),
                finish_reason: 3,
                safety_ratings: vec![],
                citation_metadata: None,
                token_count: 0,
                grounding_attributions: vec![],
                grounding_metadata: None,
                avg_logprobs: 0.0,
                logprobs_result: None,
            },
            Candidate {
                index: Some(1),
                content: Some(Content::model(";r2 i1")),
                finish_reason: 4,
                safety_ratings: vec![],
                citation_metadata: None,
                token_count: 0,
                grounding_attributions: vec![],
                grounding_metadata: None,
                avg_logprobs: 0.0,
                logprobs_result: None,
            },
        ];

        let want = vec![
            Candidate {
                index: Some(2),
                content: Some(Content::model("r1 i2")),
                finish_reason: 1,
                safety_ratings: vec![],
                citation_metadata: None,
                token_count: 0,
                grounding_attributions: vec![],
                grounding_metadata: None,
                avg_logprobs: 0.0,
                logprobs_result: None,
            },
            Candidate {
                index: Some(0),
                content: Some(Content::model("r1 i0;r2 i0")),
                finish_reason: 3,
                safety_ratings: vec![],
                citation_metadata: None,
                token_count: 0,
                grounding_attributions: vec![],
                grounding_metadata: None,
                avg_logprobs: 0.0,
                logprobs_result: None,
            },
        ];

        merge_candidates(&mut c1, &c2);
        assert_eq!(c1, want);
        let mut c3 = vec![];
        merge_candidates(&mut c3, &want);
        assert_eq!(c3, want);
    }

    #[test]
    fn merge_texts() {
        struct Test {
            update: Vec<Part>,
            want: Vec<Part>,
        }

        let tests = vec![
            Test {
                update: vec![Part::text("a")],
                want: vec![Part::text("a")],
            },
            Test {
                update: vec![Part::text("a"), Part::text("b"), Part::text("c")],
                want: vec![Part::text("abc")],
            },
            Test {
                update: vec![
                    Part::blob("b1", vec![]),
                    Part::text("a"),
                    Part::text("b"),
                    Part::blob("b2", vec![]),
                    Part::text("c"),
                ],
                want: vec![
                    Part::blob("b1", vec![]),
                    Part::text("ab"),
                    Part::blob("b2", vec![]),
                    Part::text("c"),
                ],
            },
            Test {
                update: vec![
                    Part::text("a"),
                    Part::text("b"),
                    Part::blob("b1", vec![]),
                    Part::text("c"),
                    Part::text("d"),
                    Part::blob("b2", vec![]),
                ],
                want: vec![
                    Part::text("ab"),
                    Part::blob("b1", vec![]),
                    Part::text("cd"),
                    Part::blob("b2", vec![]),
                ],
            },
        ];

        for test in tests {
            assert_eq!(merge_parts(vec![], test.update), test.want)
        }
    }
}
