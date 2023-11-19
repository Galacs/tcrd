use poise::serenity_prelude as serenity;
use rand::Rng;
use sqlx::{Row, Pool, Sqlite};
use crate::{cards::card::{Rarity, Card, Type, FightCard}, Context, Error, create_card_embed, paginate_cards};


pub async fn fight_two_players(
    ctx: &Context<'_>,
    player_a_name: String,
    player_a_cards: &mut Vec<FightCard>,
    player_b_name: String,
    player_b_cards: & mut Vec<FightCard>
) -> Result<(), Error> {
    // Which card attacks first ?
    let mut player_a_attacks = rand::thread_rng().gen_bool(0.5);

    let mut turn_count = 0;
    // Fight happens now
    while !(player_a_cards.is_empty() || player_b_cards.is_empty()) {
        // Choose which card of the two players will fight
        let player_a_index = rand::thread_rng().gen_range(0..player_a_cards.len());
        let player_b_index = rand::thread_rng().gen_range(0..player_b_cards.len());
        let card_a = &mut player_a_cards[player_a_index];
        let card_b = &mut player_b_cards[player_b_index];

        let player_b_dodge = rand::thread_rng().gen_bool((card_b.defense as f32/100.0).into());
        let player_a_dodge = rand::thread_rng().gen_bool((card_a.defense as f32/100.0).into());
        if player_a_attacks {
            // Did player B dodge ?
            if player_b_dodge {
                ctx.say(format!("{player_b_name}'s {} dodged {}'s attack with a {}% chance", card_b.id, card_a.id, card_b.defense)).await?;
            } else {
                card_b.hp -= card_a.damage;
                if card_b.hp <= 0 {
                    ctx.say(format!("{player_a_name}'s {} dealt {} damage and killed Player's B {}", card_a.id, card_a.damage, card_b.id)).await?;
                    player_b_cards.remove(player_b_index);
                } else {
                    ctx.say(format!("{player_a_name}'s {} dealt {} damage to Player's B {}. It now has {} hp", card_a.id, card_a.damage, card_b.id, card_b.hp)).await?;
                }
            }
        } else {
            // Did player A dodge ?
            if player_a_dodge {
                ctx.say(format!("{player_a_name}'s {} dodged {}'s attack with a {}% chance", card_a.id, card_b.id, card_a.defense)).await?;
            } else {
                card_a.hp -= card_b.damage;
                if card_a.hp <= 0 {
                    ctx.say(format!("{player_b_name}'s {} dealt {} damage and killed Player's A {}", card_b.id, card_b.damage, card_a.id)).await?;
                    player_a_cards.remove(player_a_index);
                } else {
                    ctx.say(format!("{player_b_name}'s {} dealt {} damage to Player's A {}. It now has {} hp", card_b.id, card_b.damage, card_a.id, card_a.hp)).await?;
                }
            }
        }

        player_a_attacks = !player_a_attacks;

        turn_count += 1;
        ctx.channel_id().broadcast_typing(&ctx.serenity_context().http).await?;
        let sleep_ms = rand::thread_rng().gen_range(250..750);
        tokio::time::sleep(tokio::time::Duration::from_millis(sleep_ms)).await;
    }

    if player_a_cards.is_empty() {
        ctx.say(format!("{player_a_name} lost in {} turns", turn_count)).await?;
    } else {
        ctx.say(format!("{player_b_name} lost in {} turns", turn_count)).await?;
    }
    Ok(())
}