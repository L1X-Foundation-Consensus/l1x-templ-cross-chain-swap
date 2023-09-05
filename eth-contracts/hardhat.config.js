require("@nomicfoundation/hardhat-toolbox");


const INFURA_API_KEY = "904a9154641d44348e7fab88570219e9";
const SYSTEM_ACCOUNT_PRIVATE_KEY = "0358ee1d35463173ed50600aa10352acdd178cd112f1e1d3d55a8998f04c086f";
const SELLER_PRIVATE_KEY = "e7efc71ab1b2055a474a6593159da8a113ad7025dca27a870e9d535501f1687c";

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: "0.8.18",
  networks: {
    goerli: {
      url: `https://goerli.infura.io/v3/${INFURA_API_KEY}`,
      accounts: [SYSTEM_ACCOUNT_PRIVATE_KEY, SELLER_PRIVATE_KEY],
    },
    bscTestnet: {
        url: "https://data-seed-prebsc-1-s1.binance.org:8545/",
        accounts: [SYSTEM_ACCOUNT_PRIVATE_KEY, SELLER_PRIVATE_KEY],
    }
  },
    etherscan: {
          apiKey: {
            bscTestnet: "X6K857FHFMS7CWHPG2X2AD3I2KBCHQP3ME",
            goerli: "CNQMU2ZM1T1CBI1IY79A1EKJFIBMU8JB8M",
          },
          // url: "https://api-testnet.bscscan.com/",
          url: "https://api-goerli.etherscan.io/",
    }
};