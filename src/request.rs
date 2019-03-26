use crate::constants::*;
use log::info;
use reqwest::Client;
use std::io::Read;
use std::result;
#[derive(Debug)]
pub enum ResquestError {
    FetchError(String),
    CreateError(String),
    DeleteError(String),
}
type Result<T> = result::Result<T, ResquestError>;

pub fn fetch_file(url: &str) -> Result<String> {
    return Client::new()
        .get(url)
        .send()
        .and_then(|mut response| {
            if response.status().is_success() {
                let mut out = String::new();
                info!("FETCH : {:?} ", url.trim());
                response
                    .read_to_string(&mut out)
                    .expect("failed read response");
                return Ok(out);
            } else {
                panic!("unsuccessful request !")
            }
        })
        .map_err(|_| ResquestError::FetchError("faild fetch file ".to_owned()));
}

pub fn create_file(data: String) -> Result<String> {
    return Client::new()
        .post(URL_SERVICE)
        .body(data)
        .send()
        .and_then(|mut response| {
            if response.status().is_success() {
                let mut out = String::new();
                response
                    .read_to_string(&mut out)
                    .expect("failed read response");
                info!("CREATE : {:?} ", out);
                return Ok(out);
            } else {
                panic!("unsuccessful create request !")
            }
        })
        .map_err(|_| ResquestError::CreateError("failed create file ! ".to_owned()));
}

pub fn delete_file(url: &str) -> Result<String> {
    return Client::new()
        .delete(url)
        .send()
        .and_then(|mut response| {
            if response.status().is_success() {
                let mut out = String::new();
                response
                    .read_to_string(&mut out)
                    .expect("failed read response");
                info!("DELETE : {:?} ", url.trim());
                return Ok(out);
            } else {
                panic!("unsuccessful delete request !")
            }
        })
        .map_err(|_| ResquestError::DeleteError("failed delete file ! ".to_owned()));
}
