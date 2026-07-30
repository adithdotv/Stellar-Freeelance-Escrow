import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { WalletButton } from "./WalletButton";

const noop = () => {};

describe("WalletButton", () => {
  it("shows a connect button when disconnected", () => {
    render(<WalletButton address={null} error={null} onConnect={noop} onDisconnect={noop} />);
    expect(screen.getByRole("button", { name: /connect wallet/i })).toBeInTheDocument();
  });

  it("calls onConnect when the connect button is clicked", async () => {
    const onConnect = vi.fn();
    render(<WalletButton address={null} error={null} onConnect={onConnect} onDisconnect={noop} />);

    await userEvent.click(screen.getByRole("button", { name: /connect wallet/i }));
    expect(onConnect).toHaveBeenCalledOnce();
  });

  it("shows the shortened address and a disconnect button when connected", () => {
    const address = "GB3JUWXW4KVGFKSSUQUIPJ2WL7K5NKPYPCJ5QJIYBHBXFWXN62WZQ2B6";
    render(<WalletButton address={address} error={null} onConnect={noop} onDisconnect={noop} />);

    expect(screen.getByText("GB3J…Q2B6")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /disconnect/i })).toBeInTheDocument();
  });

  it("shows a wallet error message", () => {
    render(
      <WalletButton
        address={null}
        error="You declined the request in your wallet."
        onConnect={noop}
        onDisconnect={noop}
      />,
    );
    expect(screen.getByText(/declined the request/i)).toBeInTheDocument();
  });
});
