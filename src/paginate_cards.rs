use poise::serenity_prelude as serenity;

use crate::{cards::card::{Card, UserCard}, create_card_embed, commands::cards::create_user_card_embed};

pub async fn paginate<U, E>(
    ctx: poise::Context<'_, U, E>,
    cards: Vec<Card>,
    user_cards: Option<Vec<UserCard>>,
) -> Result<(), serenity::Error> {
    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    // Send the embed with the first page as content
    let mut current_page = 0;
    ctx.send(|b| {
        b.embed(|b| {
            match &user_cards {
                Some(user_card) => create_user_card_embed(b, cards[current_page].clone(), user_card[current_page].clone()),
                None => create_card_embed(b, cards[current_page].clone()),
            }
        }).components(|b| {
                b.create_action_row(|b| {
                    b.create_button(|b| b.custom_id(&prev_button_id).emoji('◀'))
                        .create_button(|b| b.custom_id(&next_button_id).emoji('▶'))
                })
            })
    })
    .await?;

    // Loop through incoming interactions with the navigation buttons
    while let Some(press) = serenity::CollectComponentInteraction::new(ctx)
        // We defined our button IDs to start with `ctx_id`. If they don't, some other command's
        // button was pressed
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        // Timeout when no navigation button has been pressed for 24 hours
        .timeout(std::time::Duration::from_secs(3600 * 24))
        .await
    {
        // Depending on which button was pressed, go to next or previous page
        if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= cards.len() {
                current_page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(cards.len() - 1);
        } else {
            // This is an unrelated button interaction
            continue;
        }

        // Update the message with the new page contents
        press
            .create_interaction_response(ctx, |b| {
                b.kind(serenity::InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|b| b.embed(|b| {
                        match &user_cards {
                            Some(user_card) => create_user_card_embed(b, cards[current_page].clone(), user_card[current_page].clone()),
                            None => create_card_embed(b, cards[current_page].clone()),
                        }
                    }))
            })
            .await?;
    }

    Ok(())
}