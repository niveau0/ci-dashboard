extern crate cfg_if;
extern crate wasm_bindgen;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
// use wasm_bindgen::JsCast;
use web_sys::console;
// use web_sys::{HtmlElement, Request, RequestInit, RequestMode, Response};

use futures::{future, Future};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
// use js_sys::ArrayBuffer;
// use js_sys::Promise;
use wasm_bindgen_futures::future_to_promise;
// use wasm_bindgen_futures::JsFuture;

mod dom;
mod gitlab;
mod utils;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    server: String,
    token: String,
}

#[wasm_bindgen]
pub fn run(config: &JsValue) -> Result<(), JsValue> {
    utils::set_panic_hook();

    // console::log_1(&config);

    let config: Config = config.into_serde().expect("Failed to parse config");

    let window = web_sys::window().expect("no global `window` exists");
    let _origin = window.location().origin()?;

    let document = Arc::new(window.document().expect("should have a document on window"));

    let gitlab = Arc::new(gitlab::GitLab::new(config));

    let future = gitlab.request_projects().and_then(move |projects| {
        dom::Dom::update_content(&document.clone(), &projects);

        for project in projects {
            let gitlab = gitlab.clone();
            let document = document.clone();
            let future = gitlab
                .request_pipelines(project.id)
                .and_then(move |pipelines| {
                    if pipelines.len() > 0 {
                        let project_id = project.id;
                        dom::Dom::update_project(&document, project_id, &pipelines);
                        for pipeline in &pipelines {
                            let pipeline_id = pipeline.id;
                            let document = document.clone();
                            let gitlab = gitlab.clone();
                            let future = gitlab
                                .request_pipeline_detail(project_id, pipeline_id)
                                .and_then(move |pipeline_detail| {
                                    dom::Dom::update_pipeline(
                                        &document,
                                        project_id,
                                        &pipeline_detail,
                                    );
                                    let future = gitlab
                                        .request_jobs(project_id, pipeline_id)
                                        .and_then(move |jobs| {
                                            dom::Dom::update_jobs(
                                                &document,
                                                project_id,
                                                pipeline_id,
                                                &jobs,
                                            );
                                            future::ok(JsValue::NULL)
                                        });
                                    future_to_promise(future);
                                    future::ok(JsValue::NULL)
                                });
                            future_to_promise(future);
                        }
                    }
                    future::ok(JsValue::NULL)
                });
            future_to_promise(future);
        }
        future::ok(JsValue::NULL)
    });

    future_to_promise(future);

    Ok(())
}
