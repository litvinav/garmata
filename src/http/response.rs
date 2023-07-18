use std::borrow::Cow;

use crate::GarmataError;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: String,
    pub headers: Vec<(String, String)>,
}

impl TryFrom<Cow<'_, str>> for HttpResponse {
    type Error = GarmataError;

    fn try_from(value: Cow<str>) -> Result<Self, Self::Error> {
        let header_body_split: Vec<(usize, &str)> = value.match_indices("\r\n\r\n").collect();
        let body_start_index = match header_body_split.first() {
            Some((body_start_index, _)) => body_start_index,
            None => return Err(GarmataError { reason: "could not parse http response".into() }),
        };
        let head: Vec<&str> = value[0..*body_start_index].split("\r\n").collect();
        let mut response = HttpResponse {
            status: head.first().unwrap().split(' ').nth(1).unwrap().to_string(),
            headers: vec![],
        };
        for entry in head.iter().skip(1) {
            let mut key_val = entry.split(": ");
            if let (Some(k), Some(v)) = (key_val.next(), key_val.next()) {
                response.headers.push((k.to_lowercase(), v.to_string()));
            }
        }
        Ok(response)
    }
}
