use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, COOKIE};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::types::{BookDetails, BookInfo, LibraryBook, ResourcePermissions};

const BASE_URL: &str = "https://production.fonos.dev";
const CDN_BASE_URL: &str = "https://cdn.fonos.dev";

pub struct FonosClient {
    client: reqwest::Client,
    token: String,
}

impl FonosClient {
    pub fn new(token: String) -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, token }
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Host", HeaderValue::from_static("production.fonos.dev"));
        headers.insert(
            "User-Agent",
            HeaderValue::from_static("Fonos/1741073652 CFNetwork/3826.600.41 Darwin/24.6.0"),
        );
        let auth_value = format!("Bearer {}", self.token);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).expect("Invalid token"),
        );
        headers
    }

    pub async fn list_books(&self) -> Result<Vec<BookInfo>> {
        let url = format!("{}/users/my-library", BASE_URL);
        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("Failed to fetch library")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch library: {}", response.status());
        }

        let library_books: Vec<LibraryBook> = response
            .json()
            .await
            .context("Failed to parse library response")?;

        // Fetch book details for each book to get the title
        let mut books_info = Vec::new();
        for book in library_books {
            if book.entity != "book" {
                continue;
            }
            match self.get_book_details(book.entity_id).await {
                Ok(book_details) => {
                    books_info.push(BookInfo {
                        book_id: book.entity_id,
                        title: book_details.title,
                    });
                }
                Err(e) => {
                    // Ignore 404 errors (book not found)
                    if e.to_string().contains("Book not found (404)") {
                        println!("  Skipping book ID {} (not found)", book.entity_id);
                        continue;
                    }
                    // Propagate other errors
                    return Err(e).context("Failed to fetch book details");
                }
            }
        }

        Ok(books_info)
    }

    pub async fn get_book_details(&self, entity_id: u32) -> Result<BookDetails> {
        let url = format!("{}/books/{}", BASE_URL, entity_id);
        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("Failed to fetch book details")?;

        let status = response.status();
        if !status.is_success() {
            if status.as_u16() == 404 {
                anyhow::bail!("Book not found (404): ID = {}", entity_id);
            }
            anyhow::bail!(
                "Failed to fetch book details: ID = {}, status = {}",
                entity_id,
                status
            );
        }

        let book: BookDetails = response
            .json()
            .await
            .context("Failed to parse book details")?;

        Ok(book)
    }

    pub async fn get_resource_permissions(&self, book_id: u32) -> Result<ResourcePermissions> {
        let url = format!(
            "{}/resource-permissions?entity=books&id={}&skipCached=true",
            BASE_URL, book_id
        );
        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("Failed to fetch resource permissions")?;

        if !response.status().is_success() {
            anyhow::bail!("API error: {}", response.status());
        }

        let permissions: ResourcePermissions = response
            .json()
            .await
            .context("Failed to parse resource permissions")?;

        Ok(permissions)
    }

    pub async fn download_chapter(
        &self,
        chapter_url: &str,
        permissions: &ResourcePermissions,
        output_path: &Path,
    ) -> Result<()> {
        // Construct the full CDN URL
        let cdn_url = if chapter_url.starts_with("http") {
            chapter_url.to_string()
        } else {
            format!("{}/{}", CDN_BASE_URL, chapter_url)
        };

        // Set up headers with authentication
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&permissions.cloud_cdn_signed_cookie)
                .context("Invalid cookie")?,
        );
        headers.insert(
            "clearKey",
            HeaderValue::from_str(&permissions.clear_key).context("Invalid clearKey")?,
        );
        headers.insert("authType", HeaderValue::from_static("header"));

        // Make the request
        let response = self
            .client
            .get(&cdn_url)
            .headers(headers)
            .send()
            .await
            .context("Failed to download chapter")?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed: {}", response.status());
        }

        // Download and save without separate progress bar
        let mut file = File::create(output_path)
            .await
            .context("Failed to create output file")?;

        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read chunk")?;
            file.write_all(&chunk)
                .await
                .context("Failed to write to file")?;
        }

        Ok(())
    }
}
