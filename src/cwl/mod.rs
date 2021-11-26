use std::{
    collections::HashMap,
    sync::{mpsc::Receiver, Arc, Mutex},
    time::Duration,
};

use aws_sdk_cloudwatchlogs::Client;
use indicium::simple::{Indexable, SearchIndex};
use log::{error, info};

use crate::{log_groups::filter_log_groups, overview::QueryLogRow, status_bar::StatusMessage, App};

pub(crate) enum AwsReq {
    ListLogGroups,
    RunQuery,
}

struct MyString {
    s: String,
}

impl From<&str> for MyString {
    fn from(st: &str) -> Self {
        MyString { s: st.to_string() }
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
            if let Ok(req) = rx.recv() {
                match req {
                    AwsReq::ListLogGroups => {
                        let res = client.describe_log_groups().send().await;
                        match res {
                            Ok(mut res) => {
                                {
                                    let names: Vec<String> = res
                                        .log_groups
                                        .unwrap_or(vec![])
                                        .iter()
                                        .map(|z| z.log_group_name.as_ref().unwrap().clone())
                                        .collect();
                                    let mut app_ = app.lock().unwrap();
                                    app_.log_groups.log_group_search_index = SearchIndex::default();
                                    names
                                        .iter()
                                        .map(|x| MyString::from(x.as_str()))
                                        .enumerate()
                                        .for_each(|(index, element)| {
                                            app_.log_groups.log_group_search_index.insert(&index, &element)
                                        });
                                    app_.log_groups.log_groups = names;
                                    filter_log_groups(&mut app_);
                                }
                                loop {
                                    if res.next_token.is_none() {
                                        break;
                                    }
                                    res = client
                                        .describe_log_groups()
                                        .next_token(res.next_token.as_ref().unwrap())
                                        .send()
                                        .await
                                        .unwrap();
                                    let names: Vec<String> = res
                                        .log_groups
                                        .unwrap_or(vec![])
                                        .iter()
                                        .map(|z| z.log_group_name.as_ref().unwrap().clone())
                                        .collect();

                                    {
                                        let mut app_ = app.lock().unwrap();
                                        let num_log_groups = app_.log_groups.log_groups.len();
                                        names
                                            .iter()
                                            .map(|x| MyString::from(x.as_str()))
                                            .enumerate()
                                            .for_each(|(index, element)| {
                                                app_.log_groups.log_group_search_index.insert(&(num_log_groups + index), &element)
                                            });
                                        app_.log_groups.log_groups.extend(names);
                                        filter_log_groups(&mut app_);
                                    }
                                }
                                let mut app_ = app.lock().unwrap();
                                app_.status_message = StatusMessage::info("Log groups request completed");
                            },
                            Err(err) => {
                                error!("{:?}", err);
                                let mut app_ = app.lock().unwrap();
                                app_.status_message = StatusMessage::error("Log groups request failed");
                            },
                        }
                    }
                    AwsReq::RunQuery => {
                        let (log_groups, query_string, start, end) = {
                            let mut app_ = app.lock().unwrap();
                            let log_groups = app_.log_groups.selected_log_groups.clone();
                            let (start, end) = app_.time_selector.to_timestamps();
                            app_.status_message = StatusMessage::info("Cloudwatch Insights query started");
                            (log_groups, app_.query.clone(), start, end)
                        };
                        let res = client
                            .start_query()
                            .set_log_group_names(Some(log_groups))
                            .query_string(query_string)
                            .start_time(start)
                            .end_time(end)
                            .send()
                            .await;
                        match res {
                            Ok(res) => {

                                if let Some(query_id) = res.query_id {
                                    let mut res;
                                    loop {
                                        res = client.get_query_results().query_id(query_id.clone()).send().await.unwrap();
                                        match res.status {
                                            Some(x) if x != aws_sdk_cloudwatchlogs::model::QueryStatus::Running => break,
                                            _ => {
                                                info!("query: {:?}", res);
                                                if let Some(results) = res.results {
                                                    let mut app_ = app.lock().unwrap();
                                                    app_.log_results.query_results = results.into_iter().map(|x| {
                                                        let mut map: HashMap<String, String> = x.into_iter().map(|e| (e.field.unwrap(), e.value.unwrap())).collect();
                                                        QueryLogRow {
                                                            message: map.remove("@message").unwrap(),
                                                            timestamp: map.remove("@timestamp").unwrap(),
                                                            ptr: map.remove("@ptr").unwrap(),
                                                        }
                                                    }).collect::<Vec<_>>();
                                                }
                                            },
                                        }
                                        tokio::time::sleep(Duration::from_millis(500)).await;
                                    }
                                    if let Some(results) = res.results {
                                        let mut app_ = app.lock().unwrap();
                                        app_.log_results.query_results = results.into_iter().map(|x| {
                                            let mut map: HashMap<String, String> = x.into_iter().map(|e| (e.field.unwrap(), e.value.unwrap())).collect();
                                            QueryLogRow {
                                                message: map.remove("@message").unwrap(),
                                                timestamp: map.remove("@timestamp").unwrap(),
                                                ptr: map.remove("@ptr").unwrap(),
                                            }
                                        }).collect::<Vec<_>>();
                                        app_.status_message = StatusMessage::info("Cloudwatch Insights query completed");
                                    }
                                }
                            },
                            Err(e) => {
                                error!("{:?}", e);
                                let mut app_ = app.lock().unwrap();
                                app_.status_message = StatusMessage::error("Cloudwatch Insights query failed");
                            },
                        }
                    }

                }
            }
        }
    });
}
