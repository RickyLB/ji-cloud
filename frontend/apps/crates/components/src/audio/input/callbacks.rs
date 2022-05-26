use shared::domain::module::body::Audio;

pub struct AudioInputCallbacks {
    pub on_add: Option<Box<dyn Fn(Audio)>>,
    pub on_delete: Option<Box<dyn Fn()>>,
}

impl AudioInputCallbacks {
    pub fn new(
        on_add: Option<impl Fn(Audio) + 'static>,
        on_delete: Option<impl Fn() + 'static>,
    ) -> Self {
        Self {
            on_add: on_add.map(|f| Box::new(f) as _),
            on_delete: on_delete.map(|f| Box::new(f) as _),
        }
    }
}
