import { wagmiClient } from "./web3Setup";
import "@web3modal/ui";
import { GetAccountResult, Provider } from "@wagmi/core";

declare global {
  interface Window {
    accountState: GetAccountResult<Provider>;
    wagmiClient: any;
  }
}

window.wagmiClient = wagmiClient;
