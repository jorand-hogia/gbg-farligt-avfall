use std::io::Read;
use std::fmt;
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
    let page = fetch_page(format!("{}/wps/portal/start/avfall-och-atervinning/har-lamnar-hushall-avfall/farligtavfallbilen/farligt-avfall-bilen", BASE_URL))?;
    let paging_path = find_paging_path(page)?;
    Ok(Vec::new())
}

fn fetch_page(url: String) -> Result<impl Read, UrlParserError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    fn temp_test() {
        obtain_urls();
    }

    #[test]
    fn should_find_paging_path() {
        println!("{}", env!("CARGO_MANIFEST_DIR"));
        let file = File::open(format!("{}/src/resources/test/body_with_items.html", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let expected_path = String::from("/wps/portal/start/avfall-och-atervinning/har-lamnar-hushall-avfall/farligtavfallbilen/farligt-avfall-bilen/!ut/p/z1/04_Sj9CPykssy0xPLMnMz0vMAfIjo8ziTYzcDQy9TAy9_f1MnAwcvXxd_JwM3Y3cPcz0w8EKDFCAo4FTkJGTsYGBu7-RfhTp-pFNIk4_HgVR-I0vyA0NDXVUVAQAXsfE3Q!!/dz/d5/L2dBISEvZ0FBIS9nQSEh/p0/IZ7_42G01J41KON4B0AJMDNB1G2GP2=CZ6_42G01J41KON4B0AJMDNB1G2GH6=MDfilterDirection!filterOrganisationType!filterArea=Epagination!0==/");
        assert_eq!(expected_path, find_paging_path(file).unwrap());
    }
}