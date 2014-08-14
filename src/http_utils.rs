use std::str;
use std::io::IoResult;
use url::Url;
use http::client::RequestWriter;
use http::method::Post;

pub struct HttpResponse<'a> {
  pub code: u16,
  pub body: Vec<u8>
}

impl<'a> HttpResponse<'a> {
  pub fn body_str<'a>(&'a self) -> Option<&'a str> {
    str::from_utf8(self.body.as_slice())
  }
}

pub fn post<'a>(url: &Url) -> IoResult<HttpResponse<'a>> {
  let mut req: RequestWriter = RequestWriter::new(Post, url.clone()).unwrap();
  req.headers.insert_raw("Content-Type".to_string(), b"application/json");
  req.headers.content_length = Some(0);
  make_request(req)
}

pub fn post_json<'a>(url: &Url, json: &'a str) -> IoResult<HttpResponse<'a>> {
  let mut req: RequestWriter = RequestWriter::new(Post, url.clone()).unwrap();
  req.headers.insert_raw("Content-Type".to_string(), b"application/json");
  req.headers.content_length = Some(json.len());
  try!(req.write(json.to_string().into_bytes().as_slice()));
  make_request(req)
}

fn make_request<'a>(req: RequestWriter) -> IoResult<HttpResponse<'a>> {
  match req.read_response() {
    Ok(mut resp) => match resp.read_to_end() {
      Ok(body) => {
        Ok(HttpResponse{code: resp.status.code(), body: body})
      },
      Err(e) => Err(e)
    },
    Err((_, e)) => Err(e)
  }
}
