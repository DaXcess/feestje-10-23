use crate::{
    deck::DeckReceiver,
    emoji::EmojiPack,
    matrix::{
        animations::{
            Animation, BlocksAnimation, EyesAnimation, FallingAnimation, SequenceAnimation,
            TimeAnimation,
        },
        Matrix,
    },
};

pub enum MatrixAnimation {
    Time,
    TimeNoClock,
    Eyes,
    Falling,
    Blocks,

    // Special animation that combines all other animations and switches between them
    Sequence,
}

impl MatrixAnimation {
    fn animation(&self) -> Box<dyn Animation + Send + Sync> {
        match self {
            MatrixAnimation::Time => Box::new(TimeAnimation::new(false)),
            MatrixAnimation::TimeNoClock => Box::new(TimeAnimation::new(true)),
            MatrixAnimation::Eyes => Box::new(EyesAnimation::new()),
            MatrixAnimation::Falling => Box::new(FallingAnimation::new()),
            MatrixAnimation::Blocks => Box::new(BlocksAnimation::new()),

            MatrixAnimation::Sequence => Box::new(SequenceAnimation::new()),
        }
    }
}

pub struct AppState {
    pub deck: DeckReceiver,
    pub matrix: Matrix,
    pub emojis: EmojiPack,

    pub matrix_animation: MatrixAnimation,
}

impl AppState {
    pub fn start_matrix_animation(&self) {
        self.matrix
            .set_animation(self.matrix_animation.animation())
            .ok();
    }
}
