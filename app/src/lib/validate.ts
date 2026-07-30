/** A Stellar public key: 'G' followed by 55 base32 characters. */
const STELLAR_ADDRESS = /^G[A-Z2-7]{55}$/;

export function isStellarAddress(value: string): boolean {
  return STELLAR_ADDRESS.test(value.trim());
}
