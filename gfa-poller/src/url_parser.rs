use std::io::Read;
use std::fmt;
use regex::Regex;
use ureq::get;
use select::{document, predicate};

const BASE_URL: &str = "https://goteborg.se";

pub struct UrlParserError {
    message: String,
}
impl fmt::Debug for UrlParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

pub fn obtain_urls() -> Result<Vec<String>, UrlParserError> {
    let main_page = fetch_page(&format!("{}/wps/portal/start/avfall-och-atervinning/har-lamnar-hushall-avfall/farligtavfallbilen/farligt-avfall-bilen", BASE_URL))?;
    let paging_path = find_paging_path(main_page)?;
    let mut urls: Vec<String> = Vec::new();
    for x in 0..20 {
        let paging_num: u16 = x * 30;
        let path = format_paging_path(paging_path.clone(), paging_num);
        let next_url = format!("{}{}", BASE_URL, path);
        let page = fetch_page(&next_url)?; 
        if !has_items(page)? {
            break;
        }
        urls.push(next_url);
    }
    Ok(urls)
}

fn fetch_page(url: &String) -> Result<impl Read, UrlParserError> {
    let response = get(&url).call();
    if !response.ok() {
        return Err(UrlParserError{
            message: format!("Response from {} not in 200-range", BASE_URL)
        });
    }
    Ok(response.into_reader())
}

fn find_paging_path(page: impl Read) -> Result<String, UrlParserError> {
    let doc = match document::Document::from_read(page) {
        Ok(doc) => doc,
        Err(_e) => return Err(UrlParserError{
            message: format!("Could not parse page")
        })
    };
    let node = match doc.find(predicate::Class("c-pagination__link")).into_selection().first() {
        Some(node) => node,
        None => return Err(UrlParserError{
            message: format!("Could not find paging url")
        })
    };
    return match node.attr("href") {
        Some(url) => Ok(String::from(url)),
        None => return Err(UrlParserError{
            message: format!("Could not find href attribute of expected element")
        })
    };
}

fn format_paging_path(path: String, pagination: u16) -> String {
    let re = Regex::new(r"Epagination!\d+==/").unwrap();
    let new_pagination = format!("Epagination!{}==/", pagination);
    return String::from(re.replace(&path, &new_pagination[..]));
}

fn has_items(page: impl Read) -> Result<bool, UrlParserError> {
    let doc = match document::Document::from_read(page) {
        Ok(doc) => doc,
        Err(_e) => return Err(UrlParserError{
            message: format!("Could not parse page")
        })
    };
    Ok(!doc.find(predicate::Class("c-snippet__title")).into_selection().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn should_find_paging_path() {
        let file = File::open(format!("{}/src/resources/test/body_with_items.html", env!("CARGO_MANIFEST_DIR"))).unwrap();
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
        let file = File::open(format!("{}/src/resources/test/body_with_items.html", env!("CARGO_MANIFEST_DIR"))).unwrap();
        assert_eq!(true, has_items(file).unwrap());
    }

    #[test]
    fn should_detect_body_without_items() {
        let file = File::open(format!("{}/src/resources/test/body_without_items.html", env!("CARGO_MANIFEST_DIR"))).unwrap();
        assert_eq!(false, has_items(file).unwrap());
    }
}