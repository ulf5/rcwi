use std::sync::{Arc, Mutex, mpsc::Receiver};

use aws_sdk_cloudwatchlogs::Client;
use indicium::simple::{Indexable, SearchIndex};

use crate::{App, log_groups::filter_log_groups};

pub(crate) enum AwsReq {
    ListLogGroups,
}

struct MyString {
    s: String
}

impl From<&str> for MyString {
    fn from(st: &str) -> Self {
        MyString {
            s: st.to_string()
        }
    }
}
impl Indexable for MyString {
    fn strings(&self) -> Vec<String> {
        vec![self.s.clone()]
    }
}

pub(crate) fn run(app: Arc<Mutex<App>>, rx: Receiver<AwsReq>) {
        let basic_rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        basic_rt.block_on(async {
            let shared_config = aws_config::load_from_env().await;
            let client = Client::new(&shared_config);
            loop {
                if let Ok(_) = rx.recv() {
                    let mut res = client.describe_log_groups()
                        .send().await.unwrap();
                    {
                        let names: Vec<String> = res.log_groups.unwrap_or(vec![]).iter().map(|z| z.log_group_name.as_ref().unwrap().clone()).collect();
                        let mut app_ = app.lock().unwrap();
                        app_.log_group_search_index = SearchIndex::default();
                        names.iter()
                            .map(|x| MyString::from(x.as_str()))
                            .enumerate()
                            .for_each(|(index, element)|
                                app_.log_group_search_index.insert(&index, &element)
                            );
                        app_.log_groups = names;
                        filter_log_groups(&mut app_);
                    }
                    loop {
                        if res.next_token.is_none() {
                            break;
                        }
                        res = client.describe_log_groups()
                            .next_token(res.next_token.as_ref().unwrap())
                            .send().await.unwrap();
                        let names: Vec<String> = res.log_groups.unwrap_or(vec![]).iter().map(|z| z.log_group_name.as_ref().unwrap().clone()).collect();

                        {
                            let mut app_ = app.lock().unwrap();
                            names.iter()
                                .map(|x| MyString::from(x.as_str()))
                                .enumerate()
                                .for_each(|(index, element)|
                                    app_.log_group_search_index.insert(&index, &element)
                                );
                            app_.log_groups.extend(names);
                            filter_log_groups(&mut app_);
                        }

                    }
                }
            }
        });
}
