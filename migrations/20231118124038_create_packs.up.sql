CREATE TABLE packs (
    id VARCHAR PRIMARY KEY NOT NULL,
    price INTEGER NOT NULL,
    common_chance INTEGER DEFAULT 0 NOT NULL,
    rare_chance INTEGER DEFAULT 0 NOT NULL,
    epic_chance INTEGER DEFAULT 0 NOT NULL,
    legendary_chance INTEGER DEFAULT 0 NOT NULL,
    mythic_chance INTEGER DEFAULT 0 NOT NULL,
    awakened_chance INTEGER DEFAULT 0 NOT NULL
)