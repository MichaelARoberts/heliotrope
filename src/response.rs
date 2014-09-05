use serialize::{json, Decodable, Decoder};
use serialize::json::{Object, List, I64, U64, F64, Boolean, String};
use document::{SolrDocument, SolrField, SolrString, SolrI64, SolrU64, SolrF64, SolrBoolean, SolrNull};

pub type SolrUpdateResult = Result<SolrUpdateResponse, SolrError>;
pub type SolrQueryResult = Result<SolrQueryResponse, SolrError>;

/// SolrError
pub struct SolrError {
    /// HTTP status.
    /// When failed to connect, it will be 0 (zero).
    pub status: int,
    /// Time it took to execute the request in milliseconds
    pub time: int,
    /// Detailed error message
    pub message: String
}

impl<D: Decoder<E>, E> Decodable<D, E> for SolrError {
    fn decode(d: &mut D) -> Result<SolrError, E> {
        d.read_struct("root", 0, |d| {
            d.read_struct_field("error", 0, |d| {
                Ok(SolrError{
                    message: try!(d.read_struct_field("msg", 0, Decodable::decode)),
                    status: try!(d.read_struct_field("code", 1, Decodable::decode)),
                    // TODO: implement time parsing from request header
                    time: 0})
            })
        })
    }
}

/// Solr response used for update/indexing/commit operations
pub struct SolrUpdateResponse {
    /// HTTP status.
    /// When failed to connect, it will be 0 (zero).
    pub status: int,
    /// Time it took to execute the request in milliseconds
    pub time: int
}

impl<D: Decoder<E>, E> Decodable<D, E> for SolrUpdateResponse {
    fn decode(d: &mut D) -> Result<SolrUpdateResponse, E> {
        d.read_struct("root", 0, |d| {
            d.read_struct_field("responseHeader", 0, |d| {
                Ok(SolrUpdateResponse{
                    status: try!(d.read_struct_field("status", 0, Decodable::decode)),
                    time: try!(d.read_struct_field("QTime", 1, Decodable::decode))
                })
            })
        })
    }
}

/// Solr query response
#[deriving(Show)]
pub struct SolrQueryResponse {
    /// HTTP status.
    /// When failed to connect, it will be 0 (zero).
    pub status: u32,
    /// Time it took to execute the request in milliseconds
    pub time: u32,
    /// Total number of rows found.
    /// Note that this will probably be different from returned subset of rows,
    /// because Solr will always use pagination
    pub total: u64,
    /// Rows offset (zero based)
    pub start: u64,
    /// Current page of found Solr documents
    pub items: Vec<SolrDocument>
}

/* 
Example JSON of query response: 
```ignore
{
  "responseHeader": {
    "status": 0,
    "QTime": 1
  },
  "response": {
    "numFound": 57,
    "start": 0,
    "docs": [
      {
        "id": 1,
        "_version_": "1478235317501689856"
      },
      {
        "id": 3,
        "_version_": "1478235317504835584"
      }
    ]
  }
}
*/
impl SolrQueryResponse {
    /// Deserializes SolrQueryResponse from JSON string
    pub fn from_json_str(json_str: &str) -> SolrQueryResult {
        let mut response = SolrQueryResponse{status: 0, time: 0, total: 0, start: 0, items: Vec::new()};
        let mut error: String = "".to_string();
        match json::from_str(json_str) {
            Ok(json) => match json {
               Object(tree_map) => {
                    match tree_map.find(&"responseHeader".to_string()) {
                        Some(rh) => {
                            match rh.find(&"QTime".to_string()){
                                Some(time_json) => response.time = time_json.as_i64().unwrap() as u32,
                                None => error = "SolrQueryResponse JSON parsing error (responseHeader): QTime not found".to_string()
                            }
                            match rh.find(&"status".to_string()) {
                                Some(status_json) => response.status = status_json.as_u64().unwrap() as u32,
                                None => error = "SolrQueryResponse JSON parsing error (responseHeader): status not found".to_string()
                            }
                        },
                        None => error = "SolrQueryResponse JSON parsing error: responseHeader not found".to_string()

                    }
                    match tree_map.find(&"response".to_string()) {
                        Some(rs) => {
                            match rs.find(&"numFound".to_string()){
                                Some(total_json) => response.total = total_json.as_u64().unwrap(),
                                None => error = "SolrQueryResponse JSON parsing error (response): numFound not found".to_string()
                            }
                            match rs.find(&"start".to_string()) {
                                Some(start_json) => response.start = start_json.as_u64().unwrap(),
                                None => error = "SolrQueryResponse JSON parsing error (response): start not found".to_string()
                            }
                            match rs.find(&"docs".to_string()){
                                Some(docs_json) => {
                                    match docs_json {
                                        & List(ref docs) => {
                                            for doc_json in docs.iter() {
                                                match doc_json {
                                                    & Object(ref tm) => {
                                                        let mut doc = SolrDocument{fields: Vec::with_capacity(tm.len())};
                                                        for (k, json_v) in tm.iter() {
                                                            let v = match json_v {
                                                                & I64(i64) => SolrI64(i64),
                                                                & U64(u64) => SolrU64(u64),
                                                                & F64(f64) => SolrF64(f64),
                                                                & String(ref string) => SolrString(string.clone()),
                                                                & Boolean(bool) => SolrBoolean(bool),
                                                                _ => SolrNull
                                                            };
                                                            doc.fields.push(SolrField{name: k.clone(), value: v});
                                                        }
                                                        response.items.push(doc);
                                                    },
                                                    _ => error = "SolrQueryResponse JSON parsing error (response => docs): doc is not an object".to_string()
                                                }

                                            }
                                        },
                                        _ =>  error = "SolrQueryResponse JSON parsing error (response): docs is not a JSON list".to_string()
                                    }
                                },
                                None => error = "SolrQueryResponse JSON parsing error (response): docs not found".to_string()
                            }
                        },
                        None => error = "SolrQueryResponse JSON parsing error: response not found".to_string()
                    }
               },
               _ => error = "SolrQueryResponse JSON parsing error: query response is not a JSON object.".to_string()
            },
            Err(e) => error = format!("SolrQueryResponse JSON parsing error: {}", e).to_string()
        }
        if error.len() == 0 {
            Ok(response)
        } else {
            Err(SolrError{time: 0, status: 0, message: error})
        }
    }
}
