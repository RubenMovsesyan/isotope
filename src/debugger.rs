#[derive(Debug)]
pub enum Debugger {
    Model,
    Boson,
    ModelBoson,
    None,
}

impl Debugger {
    pub fn toggle_model(&self) -> Self {
        match self {
            Debugger::Model => Debugger::None,
            _ => Debugger::Model,
        }
    }

    pub fn toggle_boson(&self) -> Self {
        match self {
            Debugger::Boson => Debugger::None,
            _ => Debugger::Boson,
        }
    }

    pub fn toggle_model_boson(&self) -> Self {
        match self {
            Debugger::ModelBoson => Debugger::None,
            _ => Debugger::ModelBoson,
        }
    }
}
