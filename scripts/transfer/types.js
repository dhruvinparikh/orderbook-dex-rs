module.exports = {
  types: {
    Asset: {
      hash: "H256",
      symbol: "Vec<u8>",
      total_supply: "Balance"
    },
    OrderType: {
      _enum: ["Buy", "Sell"]
    },
    OrderStatus: {
      _enum: ["Pending", "PartialFilled", "Filled", "Canceled"]
    },
    ExchangePair: {
      hash: "H256",
      base: "H256",
      quote: "H256",
      latest_matched_price: "Option<Price>"
    },
    Price: "u128",
    LimitOrder: {
      hash: "H256",
      base: "H256",
      quote: "H256",
      owner: "AccountId",
      price: "Price",
      sell_amount: "Balance",
      buy_amount: "Balance",
      remained_sell_amount: "Balance",
      remained_buy_amount: "Balance",
      otype: "OrderType",
      status: "OrderStatus"
    },
    Dex: {
      hash: "H256",
      base: "H256",
      quote: "H256",
      buyer: "AccountId",
      seller: "AccountId",
      maker: "AccountId",
      taker: "AccountId",
      otype: "OrderType",
      price: "Price",
      base_amount: "Balance",
      quote_amount: "Balance"
    },
    OrderLinkedItem: {
      prev: "Option<Price>",
      next: "Option<Price>",
      price: "Option<Price>",
      orders: "Vec<H256>"
    }
  }
};
