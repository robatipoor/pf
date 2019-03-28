use crate::constants::*;
use crate::error::{ResponseError, Result};
use log::info;
use reqwest::Client;
use std::io::Read;

pub struct PastFile;

impl PastFile {
    pub fn fetch(url: &str) -> Result<String> {
        return Client::new()
            .get(url)
            .send()
            .and_then(|mut response| {
                if response.status().is_success() {
                    let mut out = String::new();
                    response
                        .read_to_string(&mut out)
                        .expect("failed read response");
                    info!("FETCH : {:?} ", url.trim());
                    out = out.trim().to_owned();
                    return Ok(out);
                } else {
                    panic!("unsuccessful request !")
                }
            })
            .map_err(|_| ResponseError::FetchError("faild fetch file ".to_owned()));
    }

    pub fn create(data: String) -> Result<String> {
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
                    out = out.trim().to_owned();
                    info!("CREATE : {:?} ", out);
                    return Ok(out);
                } else {
                    panic!("unsuccessful create request !")
                }
            })
            .map_err(|_| ResponseError::CreateError("failed create file ! ".to_owned()));
    }

    pub fn delete(url: &str) -> Result<String> {
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
                    out = out.trim().to_owned();
                    return Ok(out);
                } else {
                    panic!("unsuccessful delete request !")
                }
            })
            .map_err(|_| ResponseError::DeleteError("failed delete file ! ".to_owned()));
    }
}
