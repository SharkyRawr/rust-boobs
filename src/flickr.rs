extern crate bytes;
extern crate json;
extern crate reqwest;


pub const FLICKR_ENDPOINT: &str =
    "https://api.flickr.com/services/feeds/photos_public.gne?format=json&nojsoncallback=true";

pub fn get_photos_by_tags(tags: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let req_url = format!(
        "{endpoint}&tags={tags}",
        endpoint = FLICKR_ENDPOINT,
        tags = tags
    );
    let result = reqwest::blocking::get(&req_url)?;
    let result_text = result.text()?;
    let json = json::parse(&result_text)?;

    //let name = json["title"].to_string();

    let mut pics: Vec<String> = Vec::new();

    for item in json["items"].members() {
        let mut url = item["media"]["m"].as_str().unwrap().to_string();
        url = url.replace("_m", "_b");
        pics.push(url);
    }

    Ok(pics)
}

pub fn download_url_as_bytes(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let res = reqwest::blocking::get(url)?;
    Ok(res.bytes()?.to_vec())
}
