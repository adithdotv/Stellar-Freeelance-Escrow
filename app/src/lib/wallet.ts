import { Networks, StellarWalletsKit } from "@creit.tech/stellar-wallets-kit";
import { defaultModules } from "@creit.tech/stellar-wallets-kit/modules/utils";

StellarWalletsKit.init({
  network: Networks.TESTNET,
  modules: defaultModules(),
});

/**
 * Opens the wallet picker, then returns the chosen wallet's address.
 * Rejects if the user closes the modal without connecting.
 */
export async function connectWallet(): Promise<string> {
  const { address } = await StellarWalletsKit.authModal();
  return address;
}

export function disconnectWallet(): Promise<void> {
  return StellarWalletsKit.disconnect();
}

export function signTransaction(xdr: string) {
  return StellarWalletsKit.signTransaction(xdr, {
    networkPassphrase: Networks.TESTNET,
  });
}
