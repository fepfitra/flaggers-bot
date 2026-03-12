use reqwest::Client;

pub struct HttpService {
    pub client: Client,
}

impl HttpService {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(
                "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:61.0) Gecko/20100101 Firefox/61.0",
            )
            .build()
            .unwrap();
        Self { client }
    }
}

impl Default for HttpService {
    fn default() -> Self {
        Self::new()
    }
}
