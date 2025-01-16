require('dotenv').config();
console.log('STORK_API_KEY loaded:', process.env.STORK_API_KEY ? '✅' : '❌');
console.log('First 4 chars:', process.env.STORK_API_KEY?.substring(0, 4));

const WebSocket = require("ws");

const server = new WebSocket.Server({ port: 8081 });

server.on("connection", (client) => {
  const ws = new WebSocket("wss://api.jp.stork-oracle.network/evm/subscribe", {
    headers: {
      Authorization: `Basic ${process.env.STORK_API_KEY}`,
    },
    perMessageDeflate: true,
  });

  ws.on("open", () => {
    console.log("Connected to the API");

    ws.send(
      JSON.stringify({
        type: "subscribe",
        data: ["BTCUSD"],
      })
    );

    client.on("message", (message) => {
      console.log("Received: %s", message);
      ws.send(message);
    });
    ws.on("message", (message) => {
      const data = JSON.parse(message);
      console.log("Received: %s", data.type);
      const prices = [];

      console.log(data);
      if (data.type !== "oracle_prices") {
        return;
      }

      // Extract the main price
      if (data.data && data.data.BTCUSD && data.data.BTCUSD.price) {
        prices.push(parseInt(data.data.BTCUSD.price.slice(0, 10)));
      }

      client.send(JSON.stringify({prices: prices}));
    });
  });

  ws.on("close", () => {
    console.log("Disconnected from the API");
  });
});
