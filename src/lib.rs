extern crate cfg_if;
extern crate wasm_bindgen;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;

use futures::{future, Future};
use std::sync::Arc;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::future_to_promise;

mod dom;
mod gitlab;
mod utils;

const REFRESH_INTERVAL: i32 = 60000;

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

struct State {
    projects: Vec<dom::Project>,
}

impl State {
    fn new() -> Self {
        State { projects: vec![] }
    }

    fn set_projects(&mut self, projects: Vec<dom::Project>) {
        self.projects = projects;
    }
}

type AppState = Arc<Mutex<State>>;

#[wasm_bindgen]
pub fn run(config: JsValue) -> Result<(), JsValue> {
    utils::set_panic_hook();

    let state: AppState = Arc::new(Mutex::new(State::new()));
    let window = web_sys::window().expect("no global `window` exists");

    {
        let state = state.clone();
        let config = config.clone();
        let x = Box::new(move || {
            let state = state.clone();
            update(&config, state).unwrap_or({
                console::log_1(&JsValue::from("Failed to update"));
            })
        }) as Box<dyn Fn()>;

        let refresh = Closure::wrap(x);

        window.set_interval_with_callback_and_timeout_and_arguments_0(
            refresh.as_ref().unchecked_ref(),
            REFRESH_INTERVAL,
        )?;
        refresh.forget();
    }

    // console::log_1(&config);
    update(&config, state)
}

fn update(config: &JsValue, state: AppState) -> Result<(), JsValue> {
    let config: Arc<Config> = Arc::new(config.into_serde().expect("Failed to parse config"));

    let window = web_sys::window().expect("no global `window` exists");
    let _origin = window.location().origin()?;

    let document = Arc::new(window.document().expect("should have a document on window"));

    update_gitlab(document.clone(), state.clone(), config.clone());

    Ok(())
}

fn update_gitlab(document: Arc<web_sys::Document>, state: AppState, config: Arc<Config>) {
    let gitlab = Arc::new(gitlab::GitLab::new(config));
    let future = gitlab.request_projects().and_then(move |projects| {
        let guard = state.lock();
        match guard {
            Ok(mut state) => state.set_projects(projects.clone()),
            Err(err) => console::log_1(&JsValue::from(format!("Failed to store state {}", err))),
        };

        for project in projects {
            let gitlab = gitlab.clone();
            let document = document.clone();
            let future = gitlab
                .request_pipelines(project.id)
                .and_then(move |pipelines| {
                    if pipelines.len() > 0 {
                        let project_id = project.id;
                        let project_name = project.name;
                        dom::Dom::update_project(&document, project_id, &project_name, &pipelines);

                        let mut max = 5;
                        for pipeline in &pipelines {
                            if max < 1 {
                                break;
                            }
                            max -= 1;

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
}
