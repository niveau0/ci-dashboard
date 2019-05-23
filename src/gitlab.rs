use crate::dom;
use crate::Config;
use futures::{future, Future};
use js_sys::Promise;
use serde::Deserialize;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::console;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Deserialize, Clone)]
struct GitLabJob {
    id: i32,
    name: String,
    status: String,
    web_url: String,
}

#[derive(Deserialize)]
struct GitLabPipelineDetail {
    id: i32,
    status: String,
    r#ref: String,
    duration: Option<i32>,
}

#[derive(Deserialize)]
struct GitLabPipeline {
    id: i32,
    status: String,
}

#[derive(Deserialize)]
struct GitLabNameSpace {
    name: String,
}

#[derive(Deserialize)]
struct GitLabProject {
    id: i32,
    name: String,
    namespace: GitLabNameSpace,
}

pub struct GitLab {
    config: Arc<Config>,
}

impl GitLab {
    pub fn new(config: Arc<Config>) -> Self {
        GitLab { config }
    }

    fn set_request_headers(&self, request: &Request) {
        let headers = request.headers();
        headers
            .set("Accept", "application/json")
            .expect("Failed to set accept header");
        headers
            .set("Private-Token", &self.config.token)
            .expect("Failed to set auth header");
    }

    fn prepare_request(&self, url: &str) -> impl Future<Item = JsValue, Error = JsValue> {
        let window = web_sys::window().expect("no global `window` exists");

        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let request =
            Request::new_with_str_and_init(url, &opts).expect("Failed to initialize request");
        self.set_request_headers(&request);

        let request_promise = window.fetch_with_request(&request);

        JsFuture::from(request_promise)
            .and_then(|resp_value| {
                // `resp_value` should be a `Response` object.
                assert!(resp_value.is_instance_of::<Response>());
                resp_value.dyn_into().map(|r: Response| r.json())
            })
            .and_then(|unwrapped| unwrapped)
            .and_then(|json_value: Promise| {
                // Convert this other `Promise` into a rust `Future`.
                JsFuture::from(json_value)
            })
    }

    pub fn request_projects(&self) -> impl Future<Item = Vec<dom::Project>, Error = JsValue> {
        // console::log_1(&JsValue::from("Request projects"));
        let url = format!("{}/api/v4/projects?membership=true", self.config.server);
        let request_future = self
            .prepare_request(&url)
            .and_then(move |jsvalue| {
                jsvalue.into_serde::<Vec<GitLabProject>>().map_err(|e| {
                    JsValue::from(&format!(
                        "Failed to parse response for {}: {}",
                        &url,
                        e.to_string()
                    ))
                })
            })
            .and_then(|projects| {
                future::ok(
                    projects
                        .into_iter()
                        .map(|p| dom::Project {
                            id: p.id,
                            name: p.name,
                            group: p.namespace.name,
                        })
                        .collect::<Vec<dom::Project>>(),
                )
            });
        request_future
    }

    pub fn request_pipelines(
        &self,
        project_id: i32,
    ) -> impl Future<Item = Vec<dom::Pipeline>, Error = JsValue> {
        // console::log_1(&JsValue::from(&format!(
        //     "Request pipelines for project {}",
        //     project_id
        // )));
        let url = format!(
            "{}/api/v4/projects/{}/pipelines?order_by=id&sort=desc",
            self.config.server, project_id
        );
        let request_future = self
            .prepare_request(&url)
            .and_then(move |jsvalue| {
                jsvalue.into_serde::<Vec<GitLabPipeline>>().map_err(|e| {
                    JsValue::from(&format!(
                        "Failed to parse response for {}: {}",
                        &url,
                        e.to_string()
                    ))
                })
            })
            .and_then(|projects| {
                future::ok(
                    projects
                        .into_iter()
                        .map(|p| dom::Pipeline {
                            id: p.id,
                            status: map_status(&p.status),
                        })
                        .collect::<Vec<dom::Pipeline>>(),
                )
            });
        request_future
    }

    pub fn request_pipeline_detail(
        &self,
        project_id: i32,
        pipeline_id: i32,
    ) -> impl Future<Item = dom::PipelineDetail, Error = JsValue> {
        // console::log_1(&JsValue::from(&format!(
        //     "Request pipeline detail for project {}, pipeline {}",
        //     project_id, pipeline_id
        // )));
        let url = format!(
            "{}/api/v4/projects/{}/pipelines/{}",
            self.config.server, project_id, pipeline_id
        );
        let request_future = self
            .prepare_request(&url)
            .and_then(move |jsvalue| {
                jsvalue.into_serde::<GitLabPipelineDetail>().map_err(|e| {
                    JsValue::from(&format!(
                        "Failed to parse response for {}: {}",
                        &url,
                        e.to_string()
                    ))
                })
            })
            .and_then(|pipeline| {
                future::ok(dom::PipelineDetail {
                    id: pipeline.id,
                    status: map_status(&pipeline.status),
                    r#ref: pipeline.r#ref,
                    duration: pipeline.duration.unwrap_or(0),
                })
            });
        request_future
    }

    pub fn request_jobs(
        &self,
        project_id: i32,
        pipeline_id: i32,
    ) -> impl Future<Item = Vec<dom::Job>, Error = JsValue> {
        // console::log_1(&JsValue::from(&format!(
        //     "Request jobs for project {}, pipeline {}",
        //     project_id, pipeline_id
        // )));
        let url = format!(
            "{}/api/v4/projects/{}/pipelines/{}/jobs",
            self.config.server, project_id, pipeline_id
        );
        let request_future = self
            .prepare_request(&url)
            .and_then(move |jsvalue| {
                jsvalue.into_serde::<Vec<GitLabJob>>().map_err(|e| {
                    JsValue::from(&format!(
                        "Failed to parse response for {}: {}",
                        &url,
                        e.to_string()
                    ))
                })
            })
            .and_then(|jobs| {
                future::ok(
                    jobs
                        .into_iter()
                        .map(|j| dom::Job {
                            name: j.name,
                            status: map_status(&j.status),
                            link: j.web_url,
                        })
                        .collect::<Vec<dom::Job>>(),
                )
            });
        request_future
    }
}

fn map_status(status: &str) -> dom::Status {
    match status {
        "created" => dom::Status::CREATED,
        "pending" => dom::Status::PENDING,
        "running" => dom::Status::RUNNING,
        "failed" => dom::Status::FAILED,
        "success" => dom::Status::SUCCESS,
        "canceled" => dom::Status::CANCELED,
        "skipped" => dom::Status::SKIPPED,
        "manual" => dom::Status::MANUAL,
        _ => dom::Status::FAILED, // TODO unknown status
    }
}
