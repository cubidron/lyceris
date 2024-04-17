pub trait Reporter : Clone + Send + Sync{
    fn send(&self, case : Case);
}

pub trait Progress : Reporter{
    fn set_message(&self, message : String){
        self.send(Case::SetMessage(message))
    }
    fn set_sub_message(&self, message : String){
        self.send(Case::SetSubMessage(message))
    }
    fn set_max_progress(&self, value: f64) {
        self.send(Case::SetMaxProgress(value));
    }
    fn set_max_sub_progress(&self, value: f64) {
        self.send(Case::SetMaxSubProgress(value));
    }
    fn add_max_progress(&self, value: f64) {
        self.send(Case::AddMaxProgress(value));
    }
    fn add_max_sub_progress(&self, value: f64) {
        self.send(Case::AddMaxSubProgress(value));
    }
    fn set_progress(&self, value: f64) {
        self.send(Case::SetProgress(value));
    }
    fn set_sub_progress(&self, value: f64) {
        self.send(Case::SetSubProgress(value));
    }
    fn add_progress(&self, value: f64) {
        self.send(Case::AddProgress(value));
    }
    fn add_sub_progress(&self, value: f64) {
        self.send(Case::AddSubProgress(value));
    }
    fn set_indeterminate_progress(&self) {
        self.send(Case::SetLoadingProgress);
    }
    fn hide_progress(&self) {
        self.send(Case::HideProgress);
    }
    fn remove_progress(self) {
        self.send(Case::RemoveProgress);
    }
}

impl<R : Reporter> Progress for R {}

pub const NR: Option<()> = None;

impl<R: Reporter> Reporter for Option<R> {
    fn send(&self, state: Case) {
        if let Some(s) = &self {
            s.send(state);
        }
    }
}
#[derive(Debug)]
pub enum Case{
    SetMessage(String),
    SetSubMessage(String),
    SetMaxProgress(f64),
    SetMaxSubProgress(f64),
    AddMaxProgress(f64),
    AddMaxSubProgress(f64),
    SetProgress(f64),
    SetSubProgress(f64),
    AddProgress(f64),
    AddSubProgress(f64),
    SetLoadingProgress,
    HideProgress,
    RemoveProgress
}