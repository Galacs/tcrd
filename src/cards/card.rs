use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
    pub id: String,
    pub extension: String,
    pub rarity: Rarity,
    pub kind: Type,
    pub description: String,
    // Card stats
    pub hp: i64,
    pub damage: i64,
    pub defense: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FightCard {
    pub id: String,
    pub hp: i64,
    pub damage: i64,
    pub defense: i64,
}

impl From<Card> for FightCard {
    fn from(card: Card) -> Self {
        Self { id: card.id, hp: card.hp, damage: card.damage, defense: card.damage }
    }
}

#[derive(Debug, Clone)]
pub struct UserCard {
    pub count: i64,
}

#[derive(Clone, Debug, poise::ChoiceParameter, sqlx::Type, Serialize, Deserialize,)]
pub enum Rarity {
    Unknown,
    Common,
    Rare,
    Epic,
    Legendary,
    Mythic,
    Awakened,
    PirateKing,
    Event,
}

#[derive(Clone, Debug, poise::ChoiceParameter, sqlx::Type, Serialize, Deserialize,)]
pub enum Type {
    Unknown,
    Attacker,
    Tank,
    Defender,
}