#[derive(Debug, Clone, Copy)]
pub struct Display {
    pub id: u32, 
    pub width: u32,
    pub height: u32,
    pub frequency: f32
}
impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Display id: {}, width: {}, height {}, frequency: {}", self.id, self.width, self.height, self.frequency) 
    }
}

impl PartialEq for Display {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}