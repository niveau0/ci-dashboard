pub enum Status {
    CREATED,
    PENDING,
    RUNNING,
    SUCCESS,
    FAILED,
    SKIPPED,
    CANCELED,
    MANUAL,
}

pub struct Job {
    pub name: String,
    pub status: Status,
    pub link: String,
}

pub struct Pipeline {
    pub id: i32,
    pub status: Status,
}

pub struct PipelineDetail {
    pub id: i32,
    pub status: Status,
    pub r#ref: String,
    pub duration: i32,
}

#[derive(Clone)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub group: String,
}

pub struct Dom {}

impl Dom {
    fn map_status_to_bg(status: &Status) -> &'static str {
        match status {
            Status::SUCCESS => "bg-success",
            Status::FAILED => "bg-fail",
            Status::RUNNING => "bg-running",
            Status::MANUAL => "bg-manual",
            Status::CREATED | Status::CANCELED | Status::SKIPPED | Status::PENDING => "bg-skipped",
        }
    }

    pub fn update_project(
        document: &web_sys::Document,
        id: i32,
        name: &str,
        group: &str,
        pipelines: &Vec<Pipeline>,
    ) {
        let element_id = format!("pr{}", id);
        let project_container = match document.get_element_by_id(&element_id) {
            Some(project_container) => project_container,
            None => {
                let content = document
                    .get_element_by_id("Content")
                    .expect("document should have content region");
                let project_container = document
                    .create_element("div")
                    .expect("Failed to create project container");
                project_container.set_class_name("project hidden");
                project_container.set_id(&element_id);
                let project_name = document
                    .create_element("h1")
                    .expect("Failed to create project name");
                project_name.set_text_content(Some(&format!("{}/{}", group, name)));
                project_container
                    .append_child(&project_name)
                    .expect("Failed to add project name");

                content
                    .append_child(&project_container)
                    .expect("Failed to add project");
                project_container
            }
        };

        if pipelines.len() > 0 {
            project_container.set_class_name(&format!(
                "project {}",
                Dom::map_status_to_bg(&pipelines[0].status)
            ));
        }
    }

    pub fn update_pipeline(document: &web_sys::Document, project_id: i32, pipeline: &Pipeline) {
        let element_id = format!("pr{}", project_id);
        let project_container = document
            .get_element_by_id(&element_id)
            .expect("Failed to find project for pipeline");

        let element_id = format!("pr{}_pl{}", project_id, pipeline.id);
        if let None = document.get_element_by_id(&element_id) {
            let pipeline_container = document
                .create_element("div")
                .expect("Failed to create pipeline container");
            pipeline_container.set_class_name("pipeline bg-skipped");
            pipeline_container.set_id(&element_id);

            let label_container = document
                .create_element("div")
                .expect("Failed to create pipeline label container");
            label_container.set_class_name("label");
            label_container.set_id(&format!("pr{}_pl{}_label", project_id, pipeline.id));

            pipeline_container
                .append_child(&label_container)
                .expect("Failed to add time element");

            let time_container = document
                .create_element("div")
                .expect("Failed to create time container");
            time_container.set_class_name("time");
            time_container.set_id(&format!("pr{}_pl{}_time", project_id, pipeline.id));

            pipeline_container
                .append_child(&time_container)
                .expect("Failed to add time element");

            project_container
                .append_child(&pipeline_container)
                .expect("Failed to add pipeline");
        };
    }

    pub fn update_pipeline_detail(
        document: &web_sys::Document,
        project_id: i32,
        pipeline: &PipelineDetail,
    ) {
        let element_id = format!("pr{}_pl{}", project_id, pipeline.id);
        let pipeline_container = match document.get_element_by_id(&element_id) {
            Some(pipeline_container) => pipeline_container,
            None => return,
        };

        pipeline_container.set_class_name(&format!(
            "pipeline {}",
            Dom::map_status_to_bg(&pipeline.status)
        ));

        let hours: i32 = pipeline.duration / 3600;
        let minutes: i32 = (pipeline.duration % 3600) / 60;
        let seconds = pipeline.duration % 60;

        let element_id = format!("pr{}_pl{}_label", project_id, pipeline.id);
        let label_container = document
            .get_element_by_id(&element_id)
            .expect("Failed to find label element");
        label_container.set_text_content(Some(&format!(
            "#{} / {}",
            pipeline.id.to_string(),
            pipeline.r#ref
        )));

        let element_id = format!("pr{}_pl{}_time", project_id, pipeline.id);
        let time_container = document
            .get_element_by_id(&element_id)
            .expect("Failed to find time element");
        time_container.set_inner_html(&format!(
            r#"<i class="fas fa-clock"></i>{:02}:{:02}:{:02}"#,
            hours, minutes, seconds
        ));
    }

    pub fn update_jobs(
        document: &web_sys::Document,
        project_id: i32,
        pipeline_id: i32,
        jobs: &Vec<Job>,
    ) {
        let element_id = format!("pr{}_pl{}", project_id, pipeline_id);
        let pipeline_container = document
            .get_element_by_id(&element_id)
            .expect("missing pipeline element");

        for job in jobs {
            let element_id = format!("pr{}_pl{}_{}", project_id, pipeline_id, &job.name);
            let job_container = match document.get_element_by_id(&element_id) {
                Some(job_container) => job_container,
                None => {
                    let job_container = document
                        .create_element("div")
                        .expect("Failed to create job container");
                    job_container.set_id(&element_id);
                    pipeline_container
                        .append_child(&job_container)
                        .expect("Failed to add job");
                    job_container
                }
            };

            match job.status {
                Status::SUCCESS => {
                    job_container.set_inner_html(&format!(
                        r#"<i class="fas fa-check"></i><a href="{}" target="_blank">{}</a>"#,
                        job.link, job.name
                    ));
                    job_container.set_class_name("job job-success")
                }
                Status::FAILED => {
                    job_container.set_inner_html(&format!(
                        r#"<i class="fas fa-times-circle"></i><a href="{}" target="_blank">{}</a>"#,
                        job.link, job.name
                    ));
                    job_container.set_class_name("job job-fail");
                }
                Status::CANCELED => {
                    job_container.set_inner_html(&format!(
                        r#"<i class="fas fa-stop-circle"></i><a href="{}" target="_blank">{}</a>"#,
                        job.link, job.name
                    ));
                    job_container.set_class_name("job job-skipped");
                }
                Status::MANUAL => {
                    job_container.set_inner_html(&format!(
                        r#"<i class="fas fa-play"></i><a href="{}" target="_blank">{}</a>"#,
                        job.link, job.name
                    ));
                    job_container.set_class_name("job job-manual");
                }
                Status::RUNNING => {
                    job_container.set_inner_html(&format!(
                        r#"<i class="fas fa-cog fa-spin"></i><a href="{}" target="_blank">{}</a>"#,
                        job.link, job.name
                    ));
                    job_container.set_class_name("job job-running");
                }
                _ => {
                    job_container.set_inner_html(&format!(
                        r#"<i class="fas fa-minus-circle"></i><a href="{}" target="_blank">{}</a>"#,
                        job.link, job.name
                    ));
                    job_container.set_class_name("job job-skipped");
                }
            }
        }
    }
}
