CREATE TABLE cards (
    id VARCHAR PRIMARY KEY NOT NULL UNIQUE,
    image_extension VARCHAR NOT NULL,
    rarity VARCHAR NOT NULL,
    kind VARCHAR NOT NULL,
    description TEXT NOT NULL,
    hp INTEGER NOT NULL,
    damage INTEGER NOT NULL,
    defense INTEGER NOT NULL
)