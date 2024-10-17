use crate::integrations::raydium::RaydiumPriceFetcher;

pub async fn refresh_sol_token_prices() -> Result<(), Box<dyn std::error::Error>> {
  let raydium_client = RaydiumPriceFetcher::new();
  match raydium_client.get_token_price_list().await {
    Ok(_) => {
      println!("Updated token prices from Raydium");
      Ok(())
    }
    Err(err) => {
      eprintln!("Error fetching token price: {:?}", err);
      Err(err.into())
    }
  }
}
