CREATE TABLE users_cards (
    user_id VARCHAR NOT NULL,
    card_id VARCHAR NOT NULL REFERENCES cards(id)
)