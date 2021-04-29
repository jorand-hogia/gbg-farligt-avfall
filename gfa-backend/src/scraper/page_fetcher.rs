use futures::{future};
use std::str::FromStr;
use std::fmt;
use regex::Regex;
use reqwest::{Client, Body};
use select::{document, predicate};
use lazy_static::lazy_static;

const BASE_URL: &str = "https://goteborg.se";

#[derive(fmt::Debug)]
pub struct PageFetcherError {
    message: String,
}
impl fmt::Display for PageFetcherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.message)
    }
}

pub async fn obtain_pages() -> Result<Vec<Vec<u8>>, PageFetcherError> {
    let client = Client::builder()
        .use_rustls_tls()
        .build()
        .unwrap();
    let main_url = format!("{}/wps/portal/start/avfall-och-atervinning/har-lamnar-hushall-avfall/farligtavfallbilen/farligt-avfall-bilen", BASE_URL);
    let main_page = fetch_page(&client, main_url).await?; 
    let total_events = find_total_items(&main_page)?;
    // let paging_path = find_paging_path(&main_page)?;
    let paging_path = "/wps/portal/start/avfall-och-atervinning/har-lamnar-hushall-avfall/farligtavfallbilen/farligt-avfall-bilen/!ut/p/z1/04_Sj9CPykssy0xPLMnMz0vMAfIjo8ziTYzcDQy9TAy9_f1MnAwcvXxd_JwM3Y3cPcz0w8EKDFCAo4FTkJGTsYGBu7-RfhTp-pFNIk4_HgVR-I2PBOo3x6k_wEg_WD9KP6ogMT0zDxwm-pGWBvoFuaGhEVUhjgC0EC9V/p0/IZ7_42G01J41KON4B0AJMDNB1G2GP2=CZ6_42G01J41KON4B0AJMDNB1G2GH6=MDfilterDirection!filterOrganisationType!filterArea=Epagination!0==/".to_owned();
    let urls = calculate_urls(&paging_path, total_events);
    let results = future::join_all(
        urls.into_iter()
            .map(|url| {
                let client = &client;
                async move {
                    fetch_page(client, url).await
                }
            })
    ).await;
    let mut pages: Vec<Vec<u8>> = Vec::new();
    for result in results {
        match result {
            Ok(page) => {
                pages.push(page);
            },
            Err(e) => return Err(PageFetcherError{
                message: format!("At least one request failed: {}", e)
            })
        }; 
    } 
    Ok(pages)
}

async fn fetch_page(client: &Client, url: String) -> Result<Vec<u8>, PageFetcherError> {
    let response = match client.get(&url).send().await {
        Ok(res) => res,
        Err(_e) => return Err(PageFetcherError{
            message: format!("Request to {} failed", url)
        })
    };
    if let false = response.status().is_success() { return Err(PageFetcherError{
        message: format!("Non-OK status code from {}: {}", url, response.status())
    })};
    let response = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(_e) => return Err(PageFetcherError{
            message: format!("Failed to read response body as bytes from: {}", url)
        })
    };
    let body = Body::from(response); // TODO: Find a better way of transforming response body into Vec<u8> (or use something else than Vec<u8> which select.rs can handle)
    let body = match body.as_bytes() {
        Some(body) => body,
        None => return Err({PageFetcherError{
            message: format!("Missing response body from: {}", url)
        }})
    };
    Ok(Vec::from(body))
}

fn find_total_items(page: &[u8]) -> Result<u16, PageFetcherError> {
    lazy_static! {
        static ref TOTAL_RE: Regex = Regex::new(r".*Hittade\s+(\d+)").unwrap();
    }
    let doc = match document::Document::from_read(page) {
        Ok(doc) => doc,
        Err(_e) => return Err(PageFetcherError{
            message: "Could not parse page".to_owned()
        })
    };
    let node = match doc.find(predicate::Class("c-result-bar"))
        .into_selection()
        .first() {
            Some(node) => node,
            None => return Err(PageFetcherError{
                message: "Could not find class c-result-bar on page".to_owned()
            })
        };
    let node = node.inner_html();
    let captures = match TOTAL_RE.captures(&node) {
        Some(cap) => cap,
        None => return Err(PageFetcherError{
            message: "Could not find pattern 'Hittade TOTAL' on page".to_owned()
        })
    };
    let total = match captures.get(1) {
        Some(cap) => cap.as_str(),
        None => return Err(PageFetcherError{
            message: "Could not find capturing group 1 when parsing total".to_owned()
        })
    };
    match u16::from_str(total) {
        Ok(total) => Ok(total),
        Err(_e) => Err(PageFetcherError{
            message: "Could not parse total as u16".to_owned()
        })
    }
}

fn calculate_urls(base_path: &str, total: u16) -> Vec::<String> {
    lazy_static! {
        static ref PAGINATION_RE: Regex = Regex::new(r"Epagination!\d+==/").unwrap();
    }
    let num_urls = (total as f32 / 30.0).ceil() as u16;
    let mut urls: Vec::<String> = Vec::new();
    for i in 0..num_urls {
        let new_pagination = format!("Epagination!{}==/", i * 30);
        urls.push(format!("{}{}", BASE_URL, PAGINATION_RE.replace(base_path, &new_pagination[..]).to_owned()));
    }
    urls
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, metadata};
    use std::io::Read;

    fn read_file(path: &str) -> Vec<u8> {
        let path = &format!("{}/src/scraper/resources/test/{}", env!("CARGO_MANIFEST_DIR"), path);
        let mut file = File::open(path).unwrap();
        let md = metadata(&path).unwrap(); 
        let mut buffer = vec![0; md.len() as usize];
        file.read(&mut buffer).unwrap();
        buffer
    }

    #[test]
    fn should_find_total_items() {
        let file = read_file("body_with_items.html");
        let total = find_total_items(&file).unwrap();
        assert_eq!(177 as u16, total);
    }

    #[test]
    fn should_calculate_urls() {
        let expected_base_path = "https://goteborg.se/wps/portal/start/avfall-och-atervinning/har-lamnar-hushall-avfall/farligtavfallbilen/farligt-avfall-bilen/!ut/p/z1/04_Sj9CPykssy0xPLMnMz0vMAfIjo8ziTYzcDQy9TAy9_f1MnAwcvXxd_JwM3Y3cPcz0w8EKDFCAo4FTkJGTsYGBu7-RfhTp-pFNIk4_HgVR-I2PBOo3x6k_wEg_WD9KP6ogMT0zDxwm-pGGpgb6BbmhoRFVIY4ARalqmA!!/dz/d5/L2dBISEvZ0FBIS9nQSEh/p0/IZ7_42G01J41KON4B0AJMDNB1G2GP2=CZ6_42G01J41KON4B0AJMDNB1G2GH6=MDfilterDirection!filterOrganisationType!";
        let expected_urls = [
            format!("{}filterArea=Epagination!0==/", expected_base_path),
            format!("{}filterArea=Epagination!30==/", expected_base_path),
            format!("{}filterArea=Epagination!60==/", expected_base_path),
            format!("{}filterArea=Epagination!90==/", expected_base_path),
            format!("{}filterArea=Epagination!120==/", expected_base_path),
            format!("{}filterArea=Epagination!150==/", expected_base_path)
        ].to_vec();
        let urls = calculate_urls(
            &String::from("/wps/portal/start/avfall-och-atervinning/har-lamnar-hushall-avfall/farligtavfallbilen/farligt-avfall-bilen/!ut/p/z1/04_Sj9CPykssy0xPLMnMz0vMAfIjo8ziTYzcDQy9TAy9_f1MnAwcvXxd_JwM3Y3cPcz0w8EKDFCAo4FTkJGTsYGBu7-RfhTp-pFNIk4_HgVR-I2PBOo3x6k_wEg_WD9KP6ogMT0zDxwm-pGGpgb6BbmhoRFVIY4ARalqmA!!/dz/d5/L2dBISEvZ0FBIS9nQSEh/p0/IZ7_42G01J41KON4B0AJMDNB1G2GP2=CZ6_42G01J41KON4B0AJMDNB1G2GH6=MDfilterDirection!filterOrganisationType!filterArea=Epagination!0==/"),
            177 as u16
        );
        assert_eq!(expected_urls, urls);
    } 
}
