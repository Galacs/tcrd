CREATE TABLE fight_queue (
    user_id VARCHAR NOT NULL,
    channel_id VARCHAR NOT NULL,
    guild_id VARCHAR NOT NULL,
    join_timestamp TIMESTAMP NOT NULL DEFAULT current_timestamp,
    card_1 VARCHAR NOT NULL REFERENCES cards(id),
    card_2 VARCHAR REFERENCES cards(id),
    card_3 VARCHAR REFERENCES cards(id),
    card_4 VARCHAR REFERENCES cards(id),
    card_5 VARCHAR REFERENCES cards(id)
);