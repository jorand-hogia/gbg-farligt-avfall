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
    // TODO: Calculate URLs before starting to make all requests
    // Parse 'total items' from main_page (.c-result-bar) and calculate e.g. with math.ceiling(total / 30)
    let paging_path = find_paging_path(main_page)?;
    let mut pages: Vec<Vec<u8>> = Vec::new();
    for x in 0..20 {
        let paging_num: u16 = x * 30;
        let path = format_paging_path(paging_path.clone(), paging_num);
        let next_url = format!("{}{}", BASE_URL, path);
        let page = fetch_page(&next_url)?; 
        if !has_items(&page)? {
            break;
        }
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

fn find_paging_path(page: Vec<u8>) -> Result<String, PageFetcherError> {
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

fn find_total_items(page: Vec<u8>) -> Result<u16, PageFetcherError> {
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

fn format_paging_path(path: String, pagination: u16) -> String {
    let re = Regex::new(r"Epagination!\d+==/").unwrap();
    let new_pagination = format!("Epagination!{}==/", pagination);
    return String::from(re.replace(&path, &new_pagination[..]));
}

fn has_items(page: &Vec<u8>) -> Result<bool, PageFetcherError> {
    let doc = match document::Document::from_read(page.as_slice()) {
        Ok(doc) => doc,
        Err(_e) => return Err(PageFetcherError{
            message: format!("Could not parse page")
        })
    };
    Ok(!doc.find(predicate::Class("c-snippet__title")).into_selection().is_empty())
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
        assert_eq!(expected_path, find_paging_path(file).unwrap());
    }

    #[test]
    fn shoult_format_paging_path() {
        let base_path = "/wps/portal/start/avfall-och-atervinning/har-lamnar-hushall-avfall/farligtavfallbilen/farligt-avfall-bilen/!ut/p/z1/04_Sj9CPykssy0xPLMnMz0vMAfIjo8ziTYzcDQy9TAy9_f1MnAwcvXxd_JwM3Y3cPcz0w8EKDFCAo4FTkJGTsYGBu7-RfhTp-pFNIk4_HgVR-I0vyA0NDXVUVAQAXsfE3Q!!/dz/d5/L2dBISEvZ0FBIS9nQSEh/p0/IZ7_42G01J41KON4B0AJMDNB1G2GP2=CZ6_42G01J41KON4B0AJMDNB1G2GH6=MDfilterDirection!filterOrganisationType!";
        let expected_path = format!("{}filterArea=Epagination!60==/", base_path);
        assert_eq!(expected_path, format_paging_path(format!("{}filterArea=Epagination!0==/", base_path), 60 as u16));
    }

    #[test]
    fn should_detect_body_with_items() {
        let file = read_file("body_with_items.html");
        assert_eq!(true, has_items(&file).unwrap());
    }

    #[test]
    fn should_detect_body_without_items() {
        let file = read_file("body_without_items.html");
        assert_eq!(false, has_items(&file).unwrap());
    }

    #[test]
    fn should_find_total_items() {
        let file = read_file("body_with_items.html");
        let total = find_total_items(file).unwrap();
        assert_eq!(177 as u16, total);
    }
}