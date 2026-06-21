#[derive(Debug)]
pub struct Team {
    pub name: String,
    pub max_slots: usize,
    pub used_slots: usize,
}

impl Team {
    pub fn new(name: String, max_slots: usize) -> Self {
        Self {
            name,
            max_slots,
            used_slots: 0,
        }
    }

    pub fn available_slots(&self) -> usize {
        self.max_slots.saturating_sub(self.used_slots)
    }

    pub fn has_available_slot(&self) -> bool {
        self.available_slots() > 0
    }

    pub fn reserve_slot(&mut self) -> bool {
        if !self.has_available_slot() {
            return false;
        }

        self.used_slots += 1;
        true
    }

    pub fn release_slot(&mut self) {
        self.used_slots = self.used_slots.saturating_sub(1);
    }
}
