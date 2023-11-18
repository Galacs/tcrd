#[derive(Debug, Clone)]
pub struct Card {
    pub id: String,
    pub rarity: Rarity,
    pub description: String,
    // Card stats
    pub hp: i32,
    pub damage: i32,
    pub defense: i32,
}

#[derive(Clone, Debug, poise::ChoiceParameter, sqlx::Type)]
pub enum Rarity {
    Unknown,
    Common,
    Rare,
    Epic,
    Legendary,
    Mythic,
    Awakened,
}