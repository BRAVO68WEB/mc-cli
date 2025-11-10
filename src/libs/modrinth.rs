use reqwest;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.modrinth.com/v2";
const USER_AGENT: &str = "BRAVO68WEB/mc-cli/0.1.0";

// Search Results Response
#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResults {
    pub hits: Vec<ProjectResult>,
    pub offset: u32,
    pub limit: u32,
    pub total_hits: u32,
}

// Project Result (Search Hit)
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectResult {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub client_side: String,
    pub server_side: String,
    pub project_type: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub color: Option<u32>,
    pub thread_id: Option<String>,
    pub monetization_status: Option<String>,
    pub project_id: String,
    pub author: String,
    pub display_categories: Vec<String>,
    pub versions: Vec<String>,
    pub follows: u32,
    pub date_created: String,
    pub date_modified: String,
    pub latest_version: Option<String>,
    pub license: String,
    pub gallery: Vec<String>,
    pub featured_gallery: Option<String>,
}

// Search Query Parameters
#[derive(Debug, Default, Serialize)]
pub struct SearchQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<String>,
}

impl SearchQuery {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }
    
    pub fn facets(mut self, facets: impl Into<String>) -> Self {
        self.facets = Some(facets.into());
        self
    }
    
    pub fn index(mut self, index: impl Into<String>) -> Self {
        self.index = Some(index.into());
        self
    }
    
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }
    
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    
    pub fn filters(mut self, filters: impl Into<String>) -> Self {
        self.filters = Some(filters.into());
        self
    }
}

// API Error
#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub description: String,
}

// Main API Client
pub struct ModrinthClient {
    client: reqwest::Client,
    base_url: String,
}

impl ModrinthClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()?;
        
        Ok(Self {
            client,
            base_url: BASE_URL.to_string(),
        })
    }
    
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
    
    /// Search for projects on Modrinth
    /// 
    /// # Arguments
    /// 
    /// * `query` - Optional SearchQuery with filters and parameters
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use mc_cli::libs::modrinth::{ModrinthClient, SearchQuery};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = ModrinthClient::new()?;
    ///     
    ///     // Simple search
    ///     let results = client.search_projects(None).await?;
    ///     
    ///     // Search with query
    ///     let query = SearchQuery::new()
    ///         .query("fabric")
    ///         .limit(10);
    ///     let results = client.search_projects(Some(query)).await?;
    ///     
    ///     println!("Found {} projects", results.total_hits);
    ///     for project in results.hits {
    ///         println!("{}: {}", project.title, project.description);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn search_projects(
        &self,
        query: Option<SearchQuery>,
    ) -> Result<SearchResults, Box<dyn std::error::Error>> {
        let url = format!("{}/search", self.base_url);
        
        let mut request = self.client.get(&url);
        
        if let Some(q) = query {
            request = request.query(&q);
        }
        
        let response = request.send().await?;
        
        if response.status().is_success() {
            let results: SearchResults = response.json().await?;
            Ok(results)
        } else {
            let error: ApiError = response.json().await?;
            Err(format!("{}: {}", error.error, error.description).into())
        }
    }
}

impl Default for ModrinthClient {
    fn default() -> Self {
        Self::new().expect("Failed to create ModrinthClient")
    }
}
