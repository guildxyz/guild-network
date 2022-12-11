import { wagmiClient } from "./web3Setup";
import { GetAccountResult, Provider } from "@wagmi/core";

declare global {
  interface Window {
    accountState: GetAccountResult<Provider>;
    wagmiClient: any;
  }
}

import("@web3modal/ui").then(() => {
  console.log("@web3modal/ui imported");
});

window.wagmiClient = wagmiClient;
