use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::fmt::Display;

pub struct ProgressStatus {
    progress: ProgressBar,
    prefix: String,
}

impl ProgressStatus {
    pub fn new(multi: &MultiProgress) -> ProgressStatus {
        let progress = multi.add(ProgressBar::new(1));
        set_style(&progress);
        ProgressStatus {
            progress,
            prefix: String::new(),
        }
    }

    pub fn add_err(&mut self, p0: &str) {
        self.progress.println(format!("err: {}", p0));
        self.progress.tick()
    }
    pub fn add_info(&mut self, p0: &str) {
        self.progress.println(format!("inf: {}", p0));
        self.progress.tick()
    }

    pub fn inc(&self) {
        self.progress.inc(1);
    }

    pub fn set_prefix<T : ToString>(&mut self, prefix: T) {
        self.prefix = prefix.to_string();
    }

    pub fn set_count(&mut self, got: u32, full_count: u32) {
        self.progress.set_length(full_count as u64);
        self.progress.set_position(got as u64);
        self.prefix = format!("getting#{}: ", self.progress.position());
        self.set_message_to_progress(&self.prefix);
    }

    pub fn set_msg_keeping_prefix<S: Display>(&self, message: S) {
        self.set_message_to_progress(&format!("{}{}", self.prefix, message));
    }

    pub fn set_message(&mut self, message: &str) {
        self.set_message_to_progress(message);
    }

    pub fn set_message_to_progress(&self, msg: &str) {
        self.progress.set_message(msg);
        self.progress.tick();
    }
}

fn set_style(progress: &ProgressBar) {
    progress.set_style(ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:40blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-"));
}
