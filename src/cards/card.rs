use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
    pub id: String,
    pub extension: String,
    pub rarity: Rarity,
    pub kind: Type,
    pub description: String,
    pub obtainable: bool,
    // Card stats
    pub hp: i32,
    pub damage: i32,
    pub defense: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FightCard {
    pub id: String,
    pub hp: i32,
    pub damage: i32,
    pub defense: i32,
}

impl From<Card> for FightCard {
    fn from(card: Card) -> Self {
        Self {
            id: card.id,
            hp: card.hp,
            damage: card.damage,
            defense: card.damage,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserCard {
    pub count: i32,
}

#[derive(Clone, Debug, poise::ChoiceParameter, sqlx::Type, Serialize, Deserialize)]
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

#[derive(Clone, Debug, poise::ChoiceParameter, sqlx::Type, Serialize, Deserialize)]
pub enum Type {
    Unknown,
    Attacker,
    Tank,
    Defender,
}

