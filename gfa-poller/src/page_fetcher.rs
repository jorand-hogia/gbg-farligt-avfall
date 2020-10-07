use std::io::Read;
use std::str::FromStr;
use std::fmt;
use regex::Regex;
use ureq::get;
use select::{document, predicate};

const BASE_URL: &str = "https://goteborg.se";

#[derive(fmt::Debug)]
pub struct PageFetcherError {
    message: String,
}
impl fmt::Display for PageFetcherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n", self.message)
    }
}

pub fn obtain_pages() -> Result<Vec<Vec<u8>>, PageFetcherError> {
    let main_page = fetch_page(&format!("{}/wps/portal/start/avfall-och-atervinning/har-lamnar-hushall-avfall/farligtavfallbilen/farligt-avfall-bilen", BASE_URL))?;
    let total_events = find_total_items(&main_page)?;
    let paging_path = find_paging_path(&main_page)?;
    let urls = calculate_urls(&paging_path, total_events);
    let mut pages: Vec<Vec<u8>> = Vec::new();
    for url in urls {
        let page = fetch_page(&url)?;
        pages.push(page);
    }
    Ok(pages)
}

fn fetch_page(url: &String) -> Result<Vec<u8>, PageFetcherError> {
    let response = get(&url).call();
    if !response.ok() {
        return Err(PageFetcherError{
            message: format!("Response from {} not in 200-range", BASE_URL)
        });
    }
    let mut page_bytes = Vec::<u8>::new();
    match response.into_reader().read_to_end(&mut page_bytes) {
        Ok(_size) => Ok(page_bytes),
        Err(_e) => return Err(PageFetcherError{
            message: format!("Could not read page")
        })
    }
}

fn find_paging_path(page: &Vec<u8>) -> Result<String, PageFetcherError> {
    let doc = match document::Document::from_read(page.as_slice()) {
        Ok(doc) => doc,
        Err(_e) => return Err(PageFetcherError{
            message: format!("Could not parse page")
        })
    };
    let node = match doc.find(predicate::Class("c-pagination__link")).into_selection().first() {
        Some(node) => node,
        None => return Err(PageFetcherError{
            message: format!("Could not find paging url")
        })
    };
    return match node.attr("href") {
        Some(url) => Ok(String::from(url)),
        None => return Err(PageFetcherError{
            message: format!("Could not find href attribute of expected element")
        })
    };
}

fn find_total_items(page: &Vec<u8>) -> Result<u16, PageFetcherError> {
    let doc = match document::Document::from_read(page.as_slice()) {
        Ok(doc) => doc,
        Err(_e) => return Err(PageFetcherError{
            message: format!("Could not parse page")
        })
    };
    let node = match doc.find(predicate::Class("c-result-bar"))
        .into_selection()
        .first() {
            Some(node) => node,
            None => return Err(PageFetcherError{
                message: format!("Could not find class c-result-bar on page")
            })
        };
    let re = Regex::new(r".*Hittade\s+(\d+)").unwrap();
    let node = node.inner_html();
    let captures = match re.captures(&node) {
        Some(cap) => cap,
        None => return Err(PageFetcherError{
            message: format!("Could not find pattern 'Hittade TOTAL' on page")
        })
    };
    let total = match captures.get(1) {
        Some(cap) => cap.as_str(),
        None => return Err(PageFetcherError{
            message: format!("Could not find capturing group 1 when parsing total")
        })
    };
    return match u16::from_str(total) {
        Ok(total) => Ok(total),
        Err(_e) => Err(PageFetcherError{
            message: format!("Could not parse total as u16")
        })
    };
}

fn calculate_urls(base_path: &String, total: u16) -> Vec::<String> {
    let num_urls = (total as f32 / 30.0).ceil() as u16;
    let re = Regex::new(r"Epagination!\d+==/").unwrap();
    let mut urls: Vec::<String> = Vec::new();
    for i in 0..num_urls {
        let new_pagination = format!("Epagination!{}==/", i * 30);
        urls.push(format!("{}{}", BASE_URL, re.replace(base_path, &new_pagination[..]).to_string()));
    }
    return urls; 
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, metadata};

    fn read_file(path: &str) -> Vec<u8> {
        let path = &format!("{}/src/resources/test/{}", env!("CARGO_MANIFEST_DIR"), path);
        let mut file = File::open(path).unwrap();
        let md = metadata(&path).unwrap(); 
        let mut buffer = vec![0; md.len() as usize];
        file.read(&mut buffer).unwrap();
        buffer
    }

    #[test]
    fn should_find_paging_path() {
        let file = read_file("body_with_items.html");
        let expected_path = String::from("/wps/portal/start/avfall-och-atervinning/har-lamnar-hushall-avfall/farligtavfallbilen/farligt-avfall-bilen/!ut/p/z1/04_Sj9CPykssy0xPLMnMz0vMAfIjo8ziTYzcDQy9TAy9_f1MnAwcvXxd_JwM3Y3cPcz0w8EKDFCAo4FTkJGTsYGBu7-RfhTp-pFNIk4_HgVR-I0vyA0NDXVUVAQAXsfE3Q!!/dz/d5/L2dBISEvZ0FBIS9nQSEh/p0/IZ7_42G01J41KON4B0AJMDNB1G2GP2=CZ6_42G01J41KON4B0AJMDNB1G2GH6=MDfilterDirection!filterOrganisationType!filterArea=Epagination!0==/");
        assert_eq!(expected_path, find_paging_path(&file).unwrap());
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

    #[test]
    fn temp() {
        obtain_pages();
    }
}