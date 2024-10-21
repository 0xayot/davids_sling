// I deally I should have used events that are emitted on price updates.

// tracks the price of a token over the past 5 minutes and then sells if the price has dropped in the past.

pub async fn default_stop_loss_strategy_solana(
  db: DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
  let tokens = tokens::Entity::find()
    .filter(tokens::Column::Chain.eq("solana".to_string()))
    .all(db)
    .await?;

  for token in tokens {
    tokio::spawn(async {
      let stop_loss_trade_orders = trade_orders::Entity::find()
        .filter(trade_orders::Column::Strategy.eq("stop_loss".to_string()))
        .filter(trade_orders::Column::ContractAddress.eq(token.contract_address.to_string()))
        .filter(trade_orders::Column::TokenId.eq(token.id))
        .filter(trade_orders::Column::CreatedBy.eq("app".to_string()))
        .filter(trade_orders::Column::Active.eq(token.id))
        .join(JoinType::LeftJoin, trade_orders::Relation::Users.def())
        .join(JoinType::LeftJoin, trade_orders::Relation::Wallets.def())
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

      let five_minutes_ago = Utc::now() - Duration::minutes(5);

      let prices = prices::Entity::find()
        .filter(prices::Column::CreatedAt.gt(five_minutes_ago))
        .all(db)
        .await?;

      if let Some(latest_price) = prices.last() {
        latest_price.price
      } else {
        None
      }

      let raydium_client = RaydiumPriceFetcher::new();

      for order in stop_loss_trade_orders {
        if order.target_price >= latest_price {
          let wallet = order.wallet;

          let user = order.user;
          // get amount to swap
          let amount = match get_token_balance(order.wa).await {
            Ok(balance) => balance.amount,
            Err(_e) => continue,
          };

          // get a quote
          let quote = match raydium_client
            .get_swap_quote(
              token.contract_address,
              "So11111111111111111111111111111111111111112",
              amount.to_string(),
              50,
            )
            .await
          {
            Ok(value) => value,
            Err(_e) => continue,
          };
          // get a swap
          let swap = match raydium_client
            .get_swap_tx(
              wallet.address,
              token.contract_address,
              "So11111111111111111111111111111111111111112",
              token.public_key,
            )
            .await
          {
            Ok(tx) => tx,
            Err(e) => continue,
          };

          // execute the swap
          // get value of token in usd and otherwize
          // TODO: use a database tx here?
          match execute_user_swap_tx(user.id, wallet.id, db, swap) {
            Ok(attempt) => {
              let record = onchain_transaction::ActiveModel {
                user_id: Set(user.id),
                wallet_id: Set(wallet.id),
                transaction_hash: Set(attempt.transaction_hash),
                chain: Set("solana".to_string()),
                source: Set(Some("raydium".to_string())),
                status: Set(Some("submitted".to_string())),
                r#type: Set(Some("swap".to_string())),
                value_native: Set(Some(0.0)),
                value_usd: Set(Some(0.0)),
                from_token: Set(Some(token.contract_address)),
                to_token: Set(Some(
                  "So11111111111111111111111111111111111111112".to_string(),
                )),

                ..Default::default()
              };
              if attempt.is_confirmed {
                record.status = Set(Some("confirmed".to_string()));
              }

              OnchainTransactions
                .insert(recode)
                .exec(&context.db)
                .await
                .map_err(|e| e.to_string())?;
            }
            Err(e) => continue,
          }

          // create a transaction record
        }
      }
    })
  }
}
