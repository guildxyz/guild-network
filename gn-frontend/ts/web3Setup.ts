import { chain, configureChains, createClient } from "@wagmi/core";
import {
  EthereumClient,
  modalConnectors,
  walletConnectProvider,
} from "@web3modal/ethereum";
import { ClientCtrl, ConfigCtrl } from "@web3modal/core";
import { watchAccount } from "@wagmi/core";

const projectId = "8e6b5ffdcbc9794bf9f4a1952578365b";
const chains = [chain.mainnet];

const { provider } = configureChains(chains, [
  walletConnectProvider({ projectId }),
]);
const wagmiClient = createClient({
  autoConnect: true,
  connectors: modalConnectors({ appName: "web3Modal", chains }),
  provider,
});

const ethereumClient = new EthereumClient(wagmiClient, chains);

ConfigCtrl.setConfig({
  projectId,
  themeMode: "light",
  themeColor: "blackWhite",
});
ClientCtrl.setEthereumClient(ethereumClient);

watchAccount((accountState) => {
  window.accountState = accountState;
  window.dispatchEvent(new Event("ACCOUNT_STATE_CHANGE"));
});

export { wagmiClient };
