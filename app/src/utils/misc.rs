#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn generate_uuid() -> String {
  Uuid::new_v4().to_string()
}

// pub fn is_pump_fun_token(contract_address: String) -> bool {}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum PriceTrend {
  Increasing,
  Decreasing,
  Stable,
  Insufficient, // For when we don't have enough data points
}
#[derive(Debug, Deserialize, Serialize)]
pub struct PriceAnalysis {
  pub trend: PriceTrend,
  pub percentage: f32,
}

pub struct PriceAnalyzer {
  min_trend_length: usize,
  // Minimum percentage change to consider as significant
  min_change_threshold: f64,
}

impl PriceAnalyzer {
  pub fn new(min_trend_length: usize, min_change_threshold: f64) -> Self {
    PriceAnalyzer {
      min_trend_length,
      min_change_threshold,
    }
  }

  pub fn analyze_trend(&self, prices: &[f64]) -> PriceTrend {
    if prices.len() < self.min_trend_length {
      return PriceTrend::Insufficient;
    }

    let mut increasing_count = 0;
    let mut decreasing_count = 0;

    // Analyze consecutive price changes
    for window in prices.windows(2) {
      let previous = window[0];
      let current = window[1];

      // Calculate percentage change
      let change_percentage = (current - previous) / previous * 100.0;

      if change_percentage.abs() >= self.min_change_threshold {
        if change_percentage > 0.0 {
          increasing_count += 1;
          decreasing_count = 0;
        } else {
          decreasing_count += 1;
          increasing_count = 0;
        }
      }

      // Check if we've found a trend
      if increasing_count >= self.min_trend_length - 1 {
        return PriceTrend::Increasing;
      }
      if decreasing_count >= self.min_trend_length - 1 {
        return PriceTrend::Decreasing;
      }
    }

    PriceTrend::Stable
  }
}
