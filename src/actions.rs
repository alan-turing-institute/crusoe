use rand::Rng;

pub enum Action {
    ProduceBerries,
    Leisure,
}

impl Action {
    pub fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        match rng.random_range(0..=1) {
            0 => Action::ProduceBerries,
            1 => Action::Leisure,
            _ => unreachable!(),
        }
    }
}
