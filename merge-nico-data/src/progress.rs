use indicatif::{MultiProgress, ProgressBar, ProgressStyle, ProgressDrawTarget};
use std::ops::Deref;

pub struct Progress<'a> {
    parent: Option<&'a Progress<'a>>,
    multi: &'a MultiProgress,
    cur: ProgressBar,
}

impl <'a> Progress<'a> {
    pub fn new(multi: &MultiProgress, len: u64) -> Progress {
        multi.set_draw_target(ProgressDrawTarget::stderr());
        let child = multi.add(ProgressBar::new(len));
        set_style(&child);
        Progress {
            parent: None,
            multi,
            cur: child,
        }
    }

    pub fn create_child(&self, len: u64) -> Progress {
        let child = self.multi.add(ProgressBar::new(len));
        set_style(&child);
        Progress {
            parent: Some(self),
            multi: self.multi,
            cur: child,
        }
    }

    pub fn inc(&self, delta: u64) {
        self.cur.inc(delta);
        let mut cur = self.parent;
        while cur.is_some() {
            let c = cur.unwrap();
            c.cur.tick();
            cur = c.parent;
        }
    }

    pub fn new_generation(&mut self, len: u64) {
        let child = self.multi.add(ProgressBar::new(len));
        set_style(&child);
        self.cur = child;
    }
}

fn set_style(progress: &ProgressBar) {
    progress.set_style(ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:40blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-"));
}

impl <'a> Deref for Progress<'a> {
    type Target = ProgressBar;

    fn deref(&self) -> &Self::Target {
        &self.cur
    }
}
