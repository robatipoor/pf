use crate::constants::*;
use crate::errors::{Error, Result};
use log::info;
use reqwest::blocking::{Client,Response};
use std::io::Read;

pub struct PastFile;

impl PastFile {
    pub fn fetch(url: &str) -> Result<String> {
        Client::new()
            .get(url)
            .send()
            .and_then(|mut response: Response| {
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
            .map_err(|_| Error::ResponseError)
    }

    pub fn create<S: Into<String>>(data: S) -> Result<String> {
        Client::new()
            .post(URL_SERVICE)
            .body(data.into())
            .send()
            .and_then(|mut response: Response| {
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
            .map_err(|_| Error::ResponseError)
    }

    pub fn delete(url: &str) -> Result<String> {
        Client::new()
            .delete(url)
            .send()
            .and_then(|mut response: Response| {
                if response.status().is_success() {
                    let mut out = String::new();
                    response
                        .read_to_string(&mut out)
                        .expect("failed read response");
                    info!("DELETE : {:?} ", url.trim());
                    return Ok(out.trim().to_owned());
                } else {
                    panic!("unsuccessful delete request !")
                }
            })
            .map_err(|_| Error::ResponseError)
    }
}
