#[derive(Debug)]
pub enum Debugger {
    Model,
    Boson,
    ModelBoson,
    ModelActivated,
    BosonActivated,
    ModelBosonActivated,
    None,
}

impl Debugger {
    pub fn toggle_model(&mut self) {
        match self {
            Debugger::ModelActivated | Debugger::Model => {
                _ = std::mem::replace(self, Debugger::None);
            }
            _ => {
                _ = std::mem::replace(self, Debugger::Model);
            }
        }
    }

    pub fn toggle_boson(&mut self) {
        match self {
            Debugger::BosonActivated | Debugger::Boson => {
                _ = std::mem::replace(self, Debugger::None);
            }
            _ => {
                _ = std::mem::replace(self, Debugger::Boson);
            }
        }
    }

    pub fn toggle_model_boson(&mut self) {
        match self {
            Debugger::ModelBosonActivated | Debugger::ModelBoson => {
                _ = std::mem::replace(self, Debugger::None);
            }
            _ => {
                _ = std::mem::replace(self, Debugger::ModelBoson);
            }
        }
    }

    pub(crate) fn set_activated(&mut self) {
        match self {
            Debugger::Boson => {
                _ = std::mem::replace(self, Debugger::BosonActivated);
            }
            Debugger::Model => {
                _ = std::mem::replace(self, Debugger::ModelActivated);
            }
            Debugger::ModelBoson => {
                _ = std::mem::replace(self, Debugger::ModelBosonActivated);
            }
            _ => {}
        }
    }
}
