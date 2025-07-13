use quick_xml::events::{self, Event};
use reqwest::{self, Response, Result};

const SERVERS_URLS: [&str; 4] = [
    "http://www.speedtest.net/speedtest-servers-static.php",
    "http://c.speedtest.net/speedtest-servers-static.php",
    "http://www.speedtest.net/speedtest-servers.php",
    "http://c.speedtest.net/speedtest-servers.php"
];

#[derive(Default)]
pub struct Server {
    id: i32,
    pub url: String,
    pub name: String,
    pub country: String,
    pub sponsor: String,
}


#[derive(Default)]
pub struct Servers {
    servers: Vec<Server>,
}

impl Servers {
    pub fn new() -> Self {
        Self { servers: Vec::new() }
    }

    fn add_server(&mut self, server: Server) {
        self.servers.push(server);
    }

    pub fn get_servers(&self) -> &Vec<Server> {
        &self.servers
    }

    pub async fn update_servers(&mut self) -> Result<()> {
        print!("EO");
        let mut response_text: String = String::new();
        let mut last_error: Option<reqwest::Error> = None;
        
        for url in SERVERS_URLS {
            match reqwest::get(url).await {
                Ok(response) => {
                    match response.text().await {
                        Ok(text) => {
                            response_text = text;
                            break;
                        }
                        Err(e) => {
                            last_error = Some(e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        } 

        if response_text.is_empty() {
            return match last_error {
                Some(error) => Err(error),
                None => panic!("No servers responded and no errors were captured"),
            };
        }

        let mut response_xml = quick_xml::Reader::from_str(&response_text);
        response_xml.config_mut().trim_text(true);

        let mut buf = Vec::new();
        
        loop {
            match response_xml.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Empty(ref e)) => {
                    let mut new_server: Server = Server::default();
                    for att in e.attributes() {
                        if let Ok(attribute) = att {
                            let key = std::str::from_utf8(attribute.key.into_inner()).unwrap();
                            let value = std::str::from_utf8(&attribute.value).unwrap();
                            match key {
                                "id" => new_server.id = value.parse().unwrap(),
                                "name" => new_server.name = value.to_owned(),
                                "url" => new_server.url = value.to_owned(),
                                "country" => new_server.country = value.to_owned(),
                                "sponsor" => new_server.sponsor = value.to_owned(),
                                _ => {}
                            }
                        }
                    }
                    self.servers.push(new_server);
                }
                Ok(quick_xml::events::Event::Eof) => {
                    break;
                }
                _ => {}
            }
            buf.clear();
        }
        
        Ok(())
    }
}
