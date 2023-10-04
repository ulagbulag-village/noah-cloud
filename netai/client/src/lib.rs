use anyhow::{bail, Result};
use ark_core::env;
use serde::{de::DeserializeOwned, Serialize};

pub struct Client {
    host: String,
    http: ::reqwest::Client,
}

impl Client {
    pub fn new(host: impl ToString) -> Result<Self> {
        Ok(Self {
            host: host.to_string(),
            http: ::reqwest::Client::builder().use_rustls_tls().build()?,
        })
    }

    pub fn with_env(key: &str) -> Result<Self> {
        env::infer::<_, String>(key).and_then(Self::new)
    }

    async fn call_json<Inputs, Outputs>(&self, inputs: &Inputs) -> Result<Outputs>
    where
        Inputs: ?Sized + Serialize,
        Outputs: DeserializeOwned,
    {
        let response = self.http.post(&self.host).json(inputs).send().await?;
        if response.status().is_success() {
            response.json().await.map_err(Into::into)
        } else {
            let error = response.text().await?;
            bail!("NetAI Error: {error}")
        }
    }
}

mod nlp {
    use anyhow::Result;

    impl super::Client {
        pub async fn call_question_answering(
            &self,
            inputs: &::netai_api::nlp::question_answering::InputsRef<'_>,
        ) -> Result<::netai_api::nlp::question_answering::Outputs> {
            self.call_json(inputs).await
        }
    }
}
